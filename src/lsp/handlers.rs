use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    CodeActionParams, Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, PublishDiagnosticsParams, Uri,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Notification as _, PublishDiagnostics,
    },
    request::{CodeActionRequest, Request as _},
};

use super::document::{ServerState, is_nushell_file};

pub fn publish_diagnostics(connection: &Connection, uri: Uri, diagnostics: Vec<Diagnostic>) {
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

pub fn handle_did_open(
    connection: &Connection,
    state: &mut ServerState,
    params: serde_json::Value,
) {
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

pub fn handle_did_change(
    connection: &Connection,
    state: &mut ServerState,
    params: serde_json::Value,
) {
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

pub fn handle_did_save(
    connection: &Connection,
    state: &mut ServerState,
    params: serde_json::Value,
) {
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

pub fn handle_did_close(
    connection: &Connection,
    state: &mut ServerState,
    params: serde_json::Value,
) {
    let Ok(params) = serde_json::from_value::<DidCloseTextDocumentParams>(params) else {
        log::warn!("Failed to parse didClose params");
        return;
    };
    let uri = params.text_document.uri;
    state.close_document(&uri);
    publish_diagnostics(connection, uri, vec![]);
}

pub fn handle_notification(
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

pub fn handle_request(connection: &Connection, state: &ServerState, request: Request) {
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
