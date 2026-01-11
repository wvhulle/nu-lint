use std::path::Path;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionParams, CodeActionProviderCapability, Diagnostic,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializeParams, PublishDiagnosticsParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, Uri, WorkDoneProgressOptions,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Notification as _, PublishDiagnostics,
    },
    request::{CodeActionRequest, Request as _},
};

use super::document::{ServerState, is_nushell_file};
use crate::{Config, config::find_config_file_from};

fn load_config_from_workspace(params: &InitializeParams) -> Config {
    let workspace_path = params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .map(|folder| Path::new(folder.uri.path().as_str()));

    workspace_path.and_then(find_config_file_from).map_or_else(
        || {
            log::debug!("No workspace root or config file found, using defaults");
            Config::default()
        },
        |path| {
            log::info!("Loading config from {}", path.display());
            Config::load_from_file(&path).unwrap_or_else(|e| {
                log::warn!("Failed to load config from {}: {e}", path.display());
                Config::default()
            })
        },
    )
}

/// Run the LSP server over stdio
pub fn run_lsp_server() {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                will_save: None,
                will_save_wait_until: None,
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(
                    lsp_types::SaveOptions {
                        include_text: Some(true),
                    },
                )),
            },
        )),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
            work_done_progress_options: WorkDoneProgressOptions::default(),
            resolve_provider: None,
        })),
        ..Default::default()
    };

    let server_capabilities_json = serde_json::to_value(server_capabilities).unwrap();
    let initialization_params = match connection.initialize(server_capabilities_json) {
        Ok(params) => params,
        Err(e) => {
            log::error!("Failed to initialize LSP connection: {e}");
            return;
        }
    };

    let Ok(params) = serde_json::from_value::<InitializeParams>(initialization_params) else {
        log::error!("Failed to parse initialization params");
        return;
    };

    let config = load_config_from_workspace(&params);
    log::info!("nu-lint LSP server initialized");

    let mut state = ServerState::new(config);

    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request).unwrap_or(false) {
                    break;
                }
                handle_request(&connection, &state, request);
            }
            Message::Notification(notification) => {
                handle_notification(&connection, &mut state, notification);
            }
            Message::Response(_) => {}
        }
    }

    if let Err(e) = io_threads.join() {
        log::error!("Error joining IO threads: {e}");
    }
}

fn publish_diagnostics(connection: &Connection, uri: Uri, diagnostics: Vec<Diagnostic>) {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };

    let notification = Notification::new(PublishDiagnostics::METHOD.to_string(), params);
    if let Err(e) = connection.sender.send(Message::Notification(notification)) {
        log::error!("Failed to send diagnostics: {e}");
    }
}

fn handle_did_open(connection: &Connection, state: &mut ServerState, params: serde_json::Value) {
    let Ok(params) = serde_json::from_value::<DidOpenTextDocumentParams>(params) else {
        log::warn!("Failed to parse didOpen params");
        return;
    };
    let uri = params.text_document.uri;
    if is_nushell_file(&uri) {
        let diagnostics = state.lint_document(&uri, &params.text_document.text);
        publish_diagnostics(connection, uri, diagnostics);
    }
}

fn handle_did_change(connection: &Connection, state: &mut ServerState, params: serde_json::Value) {
    let Ok(params) = serde_json::from_value::<DidChangeTextDocumentParams>(params) else {
        log::warn!("Failed to parse didChange params");
        return;
    };
    let uri = params.text_document.uri;
    let Some(change) = params.content_changes.into_iter().last() else {
        return;
    };
    if is_nushell_file(&uri) {
        let diagnostics = state.lint_document(&uri, &change.text);
        publish_diagnostics(connection, uri, diagnostics);
    }
}

fn handle_did_save(connection: &Connection, state: &mut ServerState, params: serde_json::Value) {
    let Ok(params) = serde_json::from_value::<DidSaveTextDocumentParams>(params) else {
        log::warn!("Failed to parse didSave params");
        return;
    };
    let uri = params.text_document.uri;

    let content = params
        .text
        .or_else(|| state.get_document(&uri).map(|d| d.content.clone()));

    let Some(content) = content else {
        log::debug!("No content available for didSave on {}", uri.path());
        return;
    };

    if is_nushell_file(&uri) {
        let diagnostics = state.lint_document(&uri, &content);
        publish_diagnostics(connection, uri, diagnostics);
    }
}

fn handle_did_close(connection: &Connection, state: &mut ServerState, params: serde_json::Value) {
    let Ok(params) = serde_json::from_value::<DidCloseTextDocumentParams>(params) else {
        log::warn!("Failed to parse didClose params");
        return;
    };
    let uri = params.text_document.uri;
    state.close_document(&uri);
    publish_diagnostics(connection, uri, vec![]);
}

fn handle_notification(
    connection: &Connection,
    state: &mut ServerState,
    notification: Notification,
) {
    match notification.method.as_str() {
        DidOpenTextDocument::METHOD => handle_did_open(connection, state, notification.params),
        DidChangeTextDocument::METHOD => handle_did_change(connection, state, notification.params),
        DidSaveTextDocument::METHOD => handle_did_save(connection, state, notification.params),
        DidCloseTextDocument::METHOD => handle_did_close(connection, state, notification.params),
        _ => {}
    }
}

fn handle_request(connection: &Connection, state: &ServerState, request: Request) {
    let result = match request.method.as_str() {
        CodeActionRequest::METHOD => {
            let Ok(params) = serde_json::from_value::<CodeActionParams>(request.params) else {
                log::warn!("Failed to parse codeAction params");
                send_response(connection, request.id, serde_json::Value::Null);
                return;
            };
            let actions = state.get_code_actions(&params.text_document.uri, params.range);
            if actions.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::to_value(actions).unwrap_or(serde_json::Value::Null)
            }
        }
        _ => serde_json::Value::Null,
    };
    send_response(connection, request.id, result);
}

fn send_response(connection: &Connection, id: RequestId, result: serde_json::Value) {
    let response = Response {
        id,
        result: Some(result),
        error: None,
    };
    if let Err(e) = connection.sender.send(Message::Response(response)) {
        log::error!("Failed to send response: {e}");
    }
}
