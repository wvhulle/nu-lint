use std::{collections::HashMap, path::Path};

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Range, TextEdit, Uri,
    WorkspaceEdit,
};

use super::{
    diagnostics::{ranges_overlap, violation_to_diagnostic},
    line_index::LineIndex,
};
use crate::{Config, LintEngine, violation::Violation};

pub struct DocumentState {
    pub content: String,
    pub line_index: LineIndex,
    pub violations: Vec<Violation>,
}

pub struct ServerState {
    engine: LintEngine,
    documents: HashMap<Uri, DocumentState>,
}

impl ServerState {
    pub fn new(config: Config) -> Self {
        Self {
            engine: LintEngine::new(config),
            documents: HashMap::new(),
        }
    }

    pub fn lint_document(&mut self, uri: &Uri, content: &str) -> Vec<Diagnostic> {
        let violations = self.engine.lint_str(content);
        let line_index = LineIndex::new(content);

        let diagnostics = violations
            .iter()
            .map(|v| violation_to_diagnostic(v, content, &line_index, uri))
            .collect();

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

        doc_state
            .violations
            .iter()
            .filter_map(|violation| {
                let fix = violation.fix.as_ref()?;
                let violation_range = doc_state.line_index.span_to_range(
                    &doc_state.content,
                    violation.span.start(),
                    violation.span.end(),
                );

                let overlaps = ranges_overlap(&range, &violation_range);
                if !overlaps {
                    return None;
                }

                let edits: Vec<TextEdit> = fix
                    .replacements
                    .iter()
                    .map(|r| TextEdit {
                        range: doc_state.line_index.span_to_range(
                            &doc_state.content,
                            r.span.start(),
                            r.span.end(),
                        ),
                        new_text: r.replacement_text.to_string(),
                    })
                    .collect();

                Some(CodeActionOrCommand::CodeAction(CodeAction {
                    title: fix.explanation.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![violation_to_diagnostic(
                        violation,
                        &doc_state.content,
                        &doc_state.line_index,
                        uri,
                    )]),
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

    pub fn get_document(&self, uri: &Uri) -> Option<&DocumentState> {
        self.documents.get(uri)
    }

    pub fn close_document(&mut self, uri: &Uri) {
        self.documents.remove(uri);
    }
}

#[must_use]
pub fn is_nushell_file(uri: &Uri) -> bool {
    Path::new(uri.path().as_str())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("nu"))
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(state.get_document(&uri).is_some());
        assert_eq!(state.get_document(&uri).unwrap().content, "let x = 5");
        let _ = diagnostics;
    }

    #[test]
    fn server_state_close_document_removes_state() {
        let config = Config::default();
        let mut state = ServerState::new(config);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        state.lint_document(&uri, "let x = 5");
        assert!(state.get_document(&uri).is_some());

        state.close_document(&uri);
        assert!(state.get_document(&uri).is_none());
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
