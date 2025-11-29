use std::{collections::HashMap, path::Path};

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    InitializeParams, NumberOrString, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, TextEdit, Uri, WorkDoneProgressOptions, WorkspaceEdit,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Notification as _, PublishDiagnostics,
    },
    request::{CodeActionRequest, Request as _},
};

use crate::{Config, LintEngine, LintLevel, violation::Violation};

#[allow(
    clippy::cast_possible_truncation,
    reason = "LSP uses u32, files larger than 4GB lines/columns are unrealistic"
)]
const fn to_lsp_u32(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

fn calculate_line_column(source: &str, offset: usize) -> (usize, usize) {
    source
        .char_indices()
        .take_while(|(pos, _)| *pos < offset)
        .fold((1, 1), |(line, column), (_, ch)| {
            if ch == '\n' {
                (line + 1, 1)
            } else {
                (line, column + 1)
            }
        })
}

fn span_to_range(source: &str, start: usize, end: usize) -> Range {
    let (line_start, column_start) = calculate_line_column(source, start);
    let (line_end, column_end) = calculate_line_column(source, end);

    Range {
        start: Position {
            line: to_lsp_u32(line_start.saturating_sub(1)),
            character: to_lsp_u32(column_start.saturating_sub(1)),
        },
        end: Position {
            line: to_lsp_u32(line_end.saturating_sub(1)),
            character: to_lsp_u32(column_end.saturating_sub(1)),
        },
    }
}

fn violation_to_diagnostic(violation: &Violation, source: &str) -> Diagnostic {
    Diagnostic {
        range: span_to_range(source, violation.span.start, violation.span.end),
        severity: Some(match violation.lint_level {
            LintLevel::Deny => DiagnosticSeverity::ERROR,
            LintLevel::Warn => DiagnosticSeverity::WARNING,
            LintLevel::Allow => DiagnosticSeverity::HINT,
        }),
        code: violation
            .rule_id
            .as_ref()
            .map(|id| NumberOrString::String(id.to_string())),
        code_description: None,
        source: Some("nu-lint".to_string()),
        message: violation.message.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

struct DocumentState {
    content: String,
    violations: Vec<Violation>,
}

struct ServerState {
    engine: LintEngine,
    documents: HashMap<Uri, DocumentState>,
}

impl ServerState {
    fn new(config: Config) -> Self {
        Self {
            engine: LintEngine::new(config),
            documents: HashMap::new(),
        }
    }

    fn lint_document(&mut self, uri: &Uri, content: &str) -> Vec<Diagnostic> {
        let violations = self.engine.lint_str(content);

        let diagnostics: Vec<Diagnostic> = violations
            .iter()
            .map(|v| violation_to_diagnostic(v, content))
            .collect();

        self.documents.insert(
            uri.clone(),
            DocumentState {
                content: content.to_string(),
                violations,
            },
        );

        diagnostics
    }

    fn get_code_actions(&self, uri: &Uri, range: Range) -> Vec<CodeActionOrCommand> {
        let Some(doc_state) = self.documents.get(uri) else {
            return vec![];
        };

        doc_state
            .violations
            .iter()
            .filter_map(|violation| {
                let fix = violation.fix.as_ref()?;
                let violation_range =
                    span_to_range(&doc_state.content, violation.span.start, violation.span.end);

                let overlaps = range.start.line <= violation_range.end.line
                    && range.end.line >= violation_range.start.line;

                if !overlaps {
                    return None;
                }

                let edits: Vec<TextEdit> = fix
                    .replacements
                    .iter()
                    .map(|r| TextEdit {
                        range: span_to_range(&doc_state.content, r.span.start, r.span.end),
                        new_text: r.replacement_text.to_string(),
                    })
                    .collect();

                Some(CodeActionOrCommand::CodeAction(CodeAction {
                    title: fix.explanation.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![violation_to_diagnostic(violation, &doc_state.content)]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(uri.clone(), edits)])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                }))
            })
            .collect()
    }

    fn close_document(&mut self, uri: &Uri) {
        self.documents.remove(uri);
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

fn is_nushell_file(uri: &Uri) -> bool {
    Path::new(uri.path().as_str())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("nu"))
}

fn handle_did_open(connection: &Connection, state: &mut ServerState, params: serde_json::Value) {
    let Ok(params) = serde_json::from_value::<DidOpenTextDocumentParams>(params) else {
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
        return;
    };
    let uri = params.text_document.uri;
    let Some(content) = params.text else {
        return;
    };
    if is_nushell_file(&uri) {
        let diagnostics = state.lint_document(&uri, &content);
        publish_diagnostics(connection, uri, diagnostics);
    }
}

fn handle_did_close(connection: &Connection, state: &mut ServerState, params: serde_json::Value) {
    let Ok(params) = serde_json::from_value::<DidCloseTextDocumentParams>(params) else {
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
        CodeActionRequest::METHOD => serde_json::from_value::<CodeActionParams>(request.params)
            .ok()
            .map(|params| state.get_code_actions(&params.text_document.uri, params.range))
            .filter(|actions| !actions.is_empty())
            .and_then(|actions| serde_json::to_value(actions).ok())
            .unwrap_or(serde_json::Value::Null),
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

/// Run the LSP server over stdio
pub fn run_lsp_server(config: Config) {
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

    if let Err(e) = serde_json::from_value::<InitializeParams>(initialization_params) {
        log::error!("Failed to parse initialization params: {e}");
        return;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_column_at_start() {
        assert_eq!(calculate_line_column("hello", 0), (1, 1));
    }

    #[test]
    fn line_column_middle_of_line() {
        assert_eq!(calculate_line_column("hello world", 6), (1, 7));
    }

    #[test]
    fn line_column_after_newline() {
        assert_eq!(calculate_line_column("hello\nworld", 6), (2, 1));
    }

    #[test]
    fn line_column_multiple_lines() {
        let source = "line1\nline2\nline3";
        assert_eq!(calculate_line_column(source, 12), (3, 1));
        assert_eq!(calculate_line_column(source, 14), (3, 3));
    }

    #[test]
    fn span_to_range_single_line() {
        let source = "let x = 5";
        let range = span_to_range(source, 4, 5);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 4);
        assert_eq!(range.end.line, 0);
        assert_eq!(range.end.character, 5);
    }

    #[test]
    fn span_to_range_multiline() {
        let source = "def foo [] {\n    bar\n}";
        let range = span_to_range(source, 0, 22);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 0);
        assert_eq!(range.end.line, 2);
    }

    #[test]
    fn is_nushell_file_with_nu_extension() {
        let uri: Uri = "file:///path/to/script.nu".parse().unwrap();
        assert!(is_nushell_file(&uri));
    }

    #[test]
    fn is_nushell_file_uppercase_extension() {
        let uri: Uri = "file:///path/to/script.NU".parse().unwrap();
        assert!(is_nushell_file(&uri));
    }

    #[test]
    fn is_nushell_file_wrong_extension() {
        let uri: Uri = "file:///path/to/script.rs".parse().unwrap();
        assert!(!is_nushell_file(&uri));
    }

    #[test]
    fn is_nushell_file_no_extension() {
        let uri: Uri = "file:///path/to/script".parse().unwrap();
        assert!(!is_nushell_file(&uri));
    }

    #[test]
    fn server_state_lint_document_stores_state() {
        let config = Config::default();
        let mut state = ServerState::new(config);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        let diagnostics = state.lint_document(&uri, "let x = 5");

        assert!(state.documents.contains_key(&uri));
        assert_eq!(state.documents[&uri].content, "let x = 5");
        // Diagnostics may or may not be empty depending on rules
        let _ = diagnostics;
    }

    #[test]
    fn server_state_close_document_removes_state() {
        let config = Config::default();
        let mut state = ServerState::new(config);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        state.lint_document(&uri, "let x = 5");
        assert!(state.documents.contains_key(&uri));

        state.close_document(&uri);
        assert!(!state.documents.contains_key(&uri));
    }

    #[test]
    fn server_state_get_code_actions_empty_for_unknown_uri() {
        let config = Config::default();
        let state = ServerState::new(config);
        let uri: Uri = "file:///unknown.nu".parse().unwrap();
        let range = Range::default();

        let actions = state.get_code_actions(&uri, range);
        assert!(actions.is_empty());
    }
}
