use std::path::{Path, PathBuf};

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, Diagnostic,
    ExecuteCommandOptions, ExecuteCommandParams, HoverProviderCapability, InitializeParams,
    PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Uri,
    notification::{
        DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument, Notification as NotificationTrait, PublishDiagnostics,
    },
    request::{CodeActionRequest, ExecuteCommand, HoverRequest, Request as RequestTrait},
};

use super::{
    actions::{DISABLE_RULE_COMMAND, execute_disable_rule},
    diagnostic::{is_nushell_language_id, is_nushell_uri},
    state::ServerState,
};
use crate::{Config, config::find_config_file_from};

fn get_workspace_root(params: &InitializeParams) -> Option<PathBuf> {
    params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .map(|folder| PathBuf::from(folder.uri.path().as_str()))
}

fn load_config_from_workspace(workspace_root: Option<&Path>) -> Config {
    workspace_root.and_then(find_config_file_from).map_or_else(
        || {
            tracing::debug!("No workspace root or config file found, using defaults");
            Config::default()
        },
        |path| {
            tracing::info!("Loading config from {}", path.display());
            Config::load_from_file(&path).unwrap_or_else(|e| {
                tracing::warn!("Failed to load config from {}: {e}", path.display());
                Config::default()
            })
        },
    )
}

fn is_config_file(uri: &Uri, workspace_root: Option<&Path>) -> bool {
    workspace_root.is_some_and(|root| Path::new(uri.path().as_str()) == root.join(".nu-lint.toml"))
}

fn reload_config_and_relint(connection: &Connection, state: &mut ServerState) {
    state.reload_config();
    for uri in state.open_document_uris() {
        if let Some(doc) = state.get_document(&uri) {
            let content = doc.content.clone();
            let diagnostics = state.lint_document(&uri, &content);
            publish_diagnostics(connection, uri, diagnostics);
        }
    }
}

pub fn run_lsp_server() {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(
                    lsp_types::SaveOptions {
                        include_text: Some(true),
                    },
                )),
                ..Default::default()
            },
        )),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![
                CodeActionKind::QUICKFIX,
                CodeActionKind::SOURCE_FIX_ALL,
            ]),
            ..Default::default()
        })),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![DISABLE_RULE_COMMAND.to_string()],
            ..Default::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        ..Default::default()
    };

    let server_capabilities_json = serde_json::to_value(server_capabilities).unwrap();
    let initialization_params = match connection.initialize(server_capabilities_json) {
        Ok(params) => params,
        Err(e) => {
            tracing::error!("Failed to initialize LSP connection: {e}");
            return;
        }
    };

    let Ok(params) = serde_json::from_value::<InitializeParams>(initialization_params) else {
        tracing::error!("Failed to parse initialization params");
        return;
    };

    let workspace_root = get_workspace_root(&params);
    let config = load_config_from_workspace(workspace_root.as_deref());
    tracing::info!("nu-lint LSP server initialized");

    let mut state = ServerState::new(config, workspace_root);

    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request).unwrap_or(false) {
                    break;
                }
                handle_request(&connection, &mut state, request);
            }
            Message::Notification(notification) => {
                handle_notification(&connection, &mut state, notification);
            }
            Message::Response(_) => {}
        }
    }

    if let Err(e) = io_threads.join() {
        tracing::error!("Error joining IO threads: {e}");
    }
}

fn publish_diagnostics(connection: &Connection, uri: Uri, diagnostics: Vec<Diagnostic>) {
    let notification = Notification::new(
        PublishDiagnostics::METHOD.to_string(),
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        },
    );
    if let Err(e) = connection.sender.send(Message::Notification(notification)) {
        tracing::error!("Failed to send diagnostics: {e}");
    }
}

fn handle_notification(connection: &Connection, state: &mut ServerState, mut notif: Notification) {
    notif = try_notif::<DidOpenTextDocument, _>(notif, |params| {
        let uri = params.text_document.uri;
        if is_nushell_language_id(&params.text_document.language_id) || is_nushell_uri(&uri) {
            let diagnostics = state.lint_document(&uri, &params.text_document.text);
            publish_diagnostics(connection, uri, diagnostics);
        }
    });

    notif = try_notif::<DidChangeTextDocument, _>(notif, |params| {
        let uri = params.text_document.uri;
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };
        if is_config_file(&uri, state.workspace_root()) {
            tracing::info!("Config file changed, reloading configuration");
            reload_config_and_relint(connection, state);
        } else if state.has_document(&uri) || is_nushell_uri(&uri) {
            let diagnostics = state.lint_document(&uri, &change.text);
            publish_diagnostics(connection, uri, diagnostics);
        }
    });

    notif = try_notif::<DidSaveTextDocument, _>(notif, |params| {
        let uri = params.text_document.uri;
        if is_config_file(&uri, state.workspace_root()) {
            tracing::info!("Config file saved, reloading configuration");
            reload_config_and_relint(connection, state);
            return;
        }
        let content = params
            .text
            .or_else(|| state.get_document(&uri).map(|d| d.content.clone()));
        if let Some(content) = content
            && (state.has_document(&uri) || is_nushell_uri(&uri))
        {
            let diagnostics = state.lint_document(&uri, &content);
            publish_diagnostics(connection, uri, diagnostics);
        }
    });

    notif = try_notif::<DidCloseTextDocument, _>(notif, |params| {
        let uri = params.text_document.uri;
        state.close_document(&uri);
        publish_diagnostics(connection, uri, vec![]);
    });

    let _ = try_notif::<DidChangeWatchedFiles, _>(notif, |params| {
        if params
            .changes
            .iter()
            .any(|change| is_config_file(&change.uri, state.workspace_root()))
        {
            tracing::info!("Config file changed (watched files), reloading configuration");
            reload_config_and_relint(connection, state);
        }
    });
}

fn try_notif<N, F>(notif: Notification, handler: F) -> Notification
where
    N: NotificationTrait,
    F: FnOnce(N::Params),
{
    match notif.extract::<N::Params>(N::METHOD) {
        Ok(params) => {
            handler(params);
            Notification::new(String::new(), ()) // consumed, return dummy
        }
        Err(ExtractError::MethodMismatch(n)) => n,
        Err(ExtractError::JsonError { method, error }) => {
            tracing::warn!("Failed to parse {method} params: {error}");
            Notification::new(String::new(), ())
        }
    }
}

fn handle_request(connection: &Connection, state: &mut ServerState, req: Request) {
    let req_id = req.id.clone();

    let result = try_req::<CodeActionRequest, _>(req, |params, _| {
        let actions = state.get_code_actions(&params.text_document.uri, params.range);
        if actions.is_empty() {
            None
        } else {
            serde_json::to_value(actions).ok()
        }
    })
    .or_else(|req| {
        try_req::<HoverRequest, _>(req, |params, _| {
            state
                .get_hover(&params.text_document_position_params)
                .and_then(|h| serde_json::to_value(h).ok())
        })
    })
    .or_else(|req| {
        try_req::<ExecuteCommand, _>(req, |params, _| {
            if params.command == DISABLE_RULE_COMMAND {
                handle_disable_rule_command(connection, state, &params);
            } else {
                tracing::warn!("Unknown command: {}", params.command);
            }
            None
        })
    });

    let value = match result {
        Ok(Some(v)) => v,
        Ok(None) | Err(_) => serde_json::Value::Null,
    };
    send_response(connection, req_id, value);
}

fn try_req<R, F>(req: Request, handler: F) -> Result<Option<serde_json::Value>, Request>
where
    R: RequestTrait,
    F: FnOnce(R::Params, RequestId) -> Option<serde_json::Value>,
{
    match req.extract::<R::Params>(R::METHOD) {
        Ok((id, params)) => Ok(handler(params, id)),
        Err(ExtractError::MethodMismatch(r)) => Err(r),
        Err(ExtractError::JsonError { method, error }) => {
            tracing::warn!("Failed to parse {method} params: {error}");
            Ok(None)
        }
    }
}

fn handle_disable_rule_command(
    connection: &Connection,
    state: &mut ServerState,
    params: &ExecuteCommandParams,
) {
    let Some(rule_id) = params.arguments.first().and_then(|v| v.as_str()) else {
        tracing::warn!("disableRule command missing rule_id argument");
        return;
    };

    match execute_disable_rule(state.workspace_root(), rule_id) {
        Ok(_) => reload_config_and_relint(connection, state),
        Err(e) => tracing::error!("Failed to disable rule: {e}"),
    }
}

fn send_response(connection: &Connection, id: RequestId, result: serde_json::Value) {
    let response = Response {
        id,
        result: Some(result),
        error: None,
    };
    if let Err(e) = connection.sender.send(Message::Response(response)) {
        tracing::error!("Failed to send response: {e}");
    }
}
