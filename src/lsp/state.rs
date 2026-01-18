use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, Range, TextEdit, Uri,
    WorkspaceEdit,
};

use super::{
    diagnostic::{
        LineIndex, extra_labels_to_hint_diagnostics, ignore_comment_edit, ranges_overlap,
        violation_to_diagnostic,
    },
    server::DISABLE_RULE_COMMAND,
};
use crate::{Config, LintEngine, config::find_config_file_from, violation::Violation};

pub struct DocumentState {
    pub content: String,
    pub line_index: LineIndex,
    pub violations: Vec<Violation>,
}

pub struct ServerState {
    engine: LintEngine,
    documents: HashMap<Uri, DocumentState>,
    workspace_root: Option<PathBuf>,
}

impl ServerState {
    pub fn new(config: Config, workspace_root: Option<PathBuf>) -> Self {
        Self {
            engine: LintEngine::new(config),
            documents: HashMap::new(),
            workspace_root,
        }
    }

    /// Reload configuration from the workspace config file
    pub fn reload_config(&mut self) {
        let config = self
            .workspace_root
            .as_ref()
            .and_then(|root| {
                find_config_file_from(root).and_then(|path| {
                    log::info!("Reloading config from {}", path.display());
                    Config::load_from_file(&path).ok()
                })
            })
            .unwrap_or_default();
        self.engine = LintEngine::new(config);
    }

    /// Get the workspace root path
    #[must_use]
    pub fn workspace_root(&self) -> Option<&Path> {
        self.workspace_root.as_deref()
    }

    pub fn lint_document(&mut self, uri: &Uri, content: &str) -> Vec<Diagnostic> {
        let violations = self.engine.lint_str(content);
        let line_index = LineIndex::new(content);

        let mut diagnostics = vec![];

        for violation in &violations {
            diagnostics.push(violation_to_diagnostic(
                violation,
                content,
                &line_index,
                uri,
            ));

            diagnostics.extend(extra_labels_to_hint_diagnostics(
                violation,
                content,
                &line_index,
            ));
        }

        self.documents.insert(
            uri.clone(),
            DocumentState {
                content: content.to_string(),
                line_index,
                violations,
            },
        );

        diagnostics
    }

    pub fn get_code_actions(&self, uri: &Uri, range: Range) -> Vec<CodeActionOrCommand> {
        let Some(doc_state) = self.documents.get(uri) else {
            return vec![];
        };

        let mut actions = Vec::new();
        let is_repl = uri.scheme().is_some_and(|s| s.as_str() == "repl");

        for violation in &doc_state.violations {
            let file_span = violation.file_span();
            let violation_range = doc_state.line_index.span_to_range(
                &doc_state.content,
                file_span.start,
                file_span.end,
            );

            if !ranges_overlap(&range, &violation_range) {
                continue;
            }

            let diagnostic =
                violation_to_diagnostic(violation, &doc_state.content, &doc_state.line_index, uri);

            // Add fix action if available
            if let Some(fix) = &violation.fix {
                let edits: Vec<TextEdit> = fix
                    .replacements
                    .iter()
                    .map(|r| {
                        let file_span = r.file_span();
                        TextEdit {
                            range: doc_state.line_index.span_to_range(
                                &doc_state.content,
                                file_span.start,
                                file_span.end,
                            ),
                            new_text: r.replacement_text.to_string(),
                        }
                    })
                    .collect();

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: fix.explanation.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(uri.clone(), edits)])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                }));
            }

            // Skip inline ignore action for REPL content (no persistent file to add comment to)
            if !is_repl
                && let Some(rule_id) = violation.rule_id.as_deref()
            {
                let edit = ignore_comment_edit(&doc_state.content, file_span.start, rule_id);
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Ignore '{rule_id}' on this line"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(false),
                    disabled: None,
                    data: None,
                }));
            }
            if let Some(rule_id) = violation.rule_id.as_deref()
                && self.workspace_root.is_some()
            {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Disable rule '{rule_id}' in .nu-lint.toml"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic]),
                    edit: None,
                    command: Some(Command {
                        title: format!("Disable rule '{rule_id}'"),
                        command: DISABLE_RULE_COMMAND.to_string(),
                        arguments: Some(vec![serde_json::Value::String(rule_id.to_string())]),
                    }),
                    is_preferred: Some(false),
                    disabled: None,
                    data: None,
                }));
            }
        }

        actions
    }

    pub fn get_document(&self, uri: &Uri) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    pub fn has_document(&self, uri: &Uri) -> bool {
        self.documents.contains_key(uri)
    }

    pub fn close_document(&mut self, uri: &Uri) {
        self.documents.remove(uri);
    }

    /// Get all currently open document URIs
    #[must_use]
    pub fn open_document_uris(&self) -> Vec<Uri> {
        self.documents.keys().cloned().collect()
    }
}
