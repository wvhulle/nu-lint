use std::{
    io,
    path::{Path, PathBuf},
};

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionParams, CodeActionProviderCapability, Diagnostic,
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, ExecuteCommandOptions,
    ExecuteCommandParams, InitializeParams, PublishDiagnosticsParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, Uri, WorkDoneProgressOptions,
    notification::{
        DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
        DidSaveTextDocument, Notification as _, PublishDiagnostics,
    },
    request::{CodeActionRequest, ExecuteCommand, Request as _},
};

use super::{
    diagnostic::{is_nushell_language_id, is_nushell_uri},
    state::ServerState,
};
use crate::{Config, config::find_config_file_from};

/// Command to disable a rule in the config file
pub const DISABLE_RULE_COMMAND: &str = "nu-lint.disableRule";

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

/// Check if a URI points to the .nu-lint.toml config file
fn is_config_file(uri: &Uri, workspace_root: Option<&Path>) -> bool {
    workspace_root.is_some_and(|root| Path::new(uri.path().as_str()) == root.join(".nu-lint.toml"))
}

/// Reload config and re-lint all open documents
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
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![DISABLE_RULE_COMMAND.to_string()],
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
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

    let workspace_root = get_workspace_root(&params);
    let config = load_config_from_workspace(workspace_root.as_deref());
    log::info!("nu-lint LSP server initialized");

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
    // Check language_id first (the LSP-correct way), fall back to extension check
    if is_nushell_language_id(&params.text_document.language_id) || is_nushell_uri(&uri) {
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

    if is_config_file(&uri, state.workspace_root()) {
        log::info!("Config file changed, reloading configuration");
        reload_config_and_relint(connection, state);
    } else if state.has_document(&uri) || is_nushell_uri(&uri) {
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

    if is_config_file(&uri, state.workspace_root()) {
        log::info!("Config file saved, reloading configuration");
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

fn handle_did_change_watched_files(
    connection: &Connection,
    state: &mut ServerState,
    params: serde_json::Value,
) {
    let Ok(params) = serde_json::from_value::<DidChangeWatchedFilesParams>(params) else {
        log::warn!("Failed to parse didChangeWatchedFiles params");
        return;
    };

    let config_changed = params
        .changes
        .iter()
        .any(|change| is_config_file(&change.uri, state.workspace_root()));

    if config_changed {
        log::info!("Config file changed (watched files), reloading configuration");
        reload_config_and_relint(connection, state);
    }
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
        DidChangeWatchedFiles::METHOD => {
            handle_did_change_watched_files(connection, state, notification.params);
        }
        _ => {}
    }
}

fn handle_request(connection: &Connection, state: &mut ServerState, request: Request) {
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
        ExecuteCommand::METHOD => {
            let Ok(params) = serde_json::from_value::<ExecuteCommandParams>(request.params) else {
                log::warn!("Failed to parse executeCommand params");
                send_response(connection, request.id, serde_json::Value::Null);
                return;
            };
            if params.command == DISABLE_RULE_COMMAND {
                handle_disable_rule_command(connection, state, &params);
            } else {
                log::warn!("Unknown command: {}", params.command);
            }
            serde_json::Value::Null
        }
        _ => serde_json::Value::Null,
    };
    send_response(connection, request.id, result);
}

fn handle_disable_rule_command(
    connection: &Connection,
    state: &mut ServerState,
    params: &ExecuteCommandParams,
) {
    let Some(rule_id) = params.arguments.first().and_then(|v| v.as_str()) else {
        log::warn!("disableRule command missing rule_id argument");
        return;
    };

    let Some(workspace_root) = state.workspace_root() else {
        log::warn!("No workspace root available for disableRule command");
        return;
    };

    let config_path = workspace_root.join(".nu-lint.toml");
    if let Err(e) = disable_rule_in_config(&config_path, rule_id) {
        log::error!("Failed to disable rule in config: {e}");
        return;
    }

    log::info!("Disabled rule '{rule_id}' in config file");
    reload_config_and_relint(connection, state);
}

/// Add `rule_id = "off"` to the config file's [rules] section.
/// Creates the file or section if needed.
fn disable_rule_in_config(config_path: &Path, rule_id: &str) -> io::Result<()> {
    use std::fs;

    let rule_entry = format!("{rule_id} = \"off\"\n");

    let content = if config_path.exists() {
        fs::read_to_string(config_path)?
    } else {
        String::new()
    };

    // Already disabled
    if content.contains(&format!("{rule_id} = \"off\"")) {
        return Ok(());
    }

    let new_content = match content.find("[rules]") {
        Some(pos) => {
            // Insert after the [rules] line
            let insert_pos = content[pos..]
                .find('\n')
                .map_or(content.len(), |p| pos + p + 1);
            format!(
                "{}{rule_entry}{}",
                &content[..insert_pos],
                &content[insert_pos..]
            )
        }
        None if content.is_empty() => {
            format!("[rules]\n{rule_entry}")
        }
        None => {
            let separator = if content.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            format!("{content}{separator}[rules]\n{rule_entry}")
        }
    };

    fs::write(config_path, new_content)
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
