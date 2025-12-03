use std::path::Path;

use lsp_server::{Connection, Message};
use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, InitializeParams,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, WorkDoneProgressOptions,
};

use super::{
    document::ServerState,
    handlers::{handle_notification, handle_request},
};
use crate::{Config, config::find_config_file_from};

fn load_config_from_workspace(params: &InitializeParams) -> Config {
    let workspace_path = params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .map(|folder| Path::new(folder.uri.path().as_str()))
        .or_else(|| {
            #[allow(
                deprecated,
                reason = "root_uri is deprecated but needed for older clients"
            )]
            params
                .root_uri
                .as_ref()
                .map(|uri| Path::new(uri.path().as_str()))
        });

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
