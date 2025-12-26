use std::{collections::HashMap, path::Path};

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeDescription, Diagnostic,
    DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Range, TextEdit,
    Uri, WorkspaceEdit,
};

use super::line_index::LineIndex;
use crate::{Config, LintEngine, LintLevel, violation::Violation};

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
                let file_span = violation.file_span();
                let violation_range = doc_state.line_index.span_to_range(
                    &doc_state.content,
                    file_span.start,
                    file_span.end,
                );

                let overlaps = ranges_overlap(&range, &violation_range);
                if !overlaps {
                    return None;
                }

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

const fn lint_level_to_severity(level: LintLevel) -> DiagnosticSeverity {
    match level {
        LintLevel::Error => DiagnosticSeverity::ERROR,
        LintLevel::Warning => DiagnosticSeverity::WARNING,
        LintLevel::Hint => DiagnosticSeverity::HINT,
    }
}

pub fn violation_to_diagnostic(
    violation: &Violation,
    source: &str,
    line_index: &LineIndex,
    file_uri: &Uri,
) -> Diagnostic {
    let message = build_message(violation);
    let related_information = build_related_information(violation, source, line_index, file_uri);
    let file_span = violation.file_span();

    Diagnostic {
        range: line_index.span_to_range(source, file_span.start, file_span.end),
        severity: Some(lint_level_to_severity(violation.lint_level)),
        code: violation
            .rule_id
            .as_deref()
            .map(|id| NumberOrString::String(id.to_string())),
        code_description: violation
            .doc_url
            .and_then(|url| url.parse().ok())
            .map(|href| CodeDescription { href }),
        source: Some(String::from("nu-lint")),
        message,
        related_information: if related_information.is_empty() {
            None
        } else {
            Some(related_information)
        },
        tags: None,
        data: None,
    }
}

fn build_message(violation: &Violation) -> String {
    let base_message = violation.primary_label.as_ref().map_or_else(
        || violation.message.to_string(),
        |label| format!("{}: {label}", violation.message),
    );

    violation
        .help
        .as_ref()
        .map_or(base_message.clone(), |help| {
            format!("{base_message}\n\nHelp: {help}")
        })
}

fn build_related_information(
    violation: &Violation,
    source: &str,
    line_index: &LineIndex,
    file_uri: &Uri,
) -> Vec<DiagnosticRelatedInformation> {
    violation
        .extra_labels
        .iter()
        .filter_map(|(span, label)| {
            let label_text = label.as_deref()?;
            if label_text.is_empty() {
                return None;
            }

            let file_span = span.file_span();
            let range = line_index.span_to_range(source, file_span.start, file_span.end);

            Some(DiagnosticRelatedInformation {
                location: Location {
                    uri: file_uri.clone(),
                    range,
                },
                message: label_text.to_string(),
            })
        })
        .collect()
}

#[must_use]
pub const fn ranges_overlap(a: &Range, b: &Range) -> bool {
    a.start.line <= b.end.line && b.start.line <= a.end.line
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

    #[test]
    fn ranges_overlap_same_line() {
        use lsp_types::Position;

        let a = Range {
            start: Position {
                line: 1,
                character: 0,
            },
            end: Position {
                line: 1,
                character: 10,
            },
        };
        let b = Range {
            start: Position {
                line: 1,
                character: 5,
            },
            end: Position {
                line: 1,
                character: 15,
            },
        };
        assert!(ranges_overlap(&a, &b));
    }

    #[test]
    fn ranges_overlap_different_lines() {
        use lsp_types::Position;

        let a = Range {
            start: Position {
                line: 1,
                character: 0,
            },
            end: Position {
                line: 3,
                character: 0,
            },
        };
        let b = Range {
            start: Position {
                line: 2,
                character: 0,
            },
            end: Position {
                line: 4,
                character: 0,
            },
        };
        assert!(ranges_overlap(&a, &b));
    }

    #[test]
    fn ranges_no_overlap() {
        use lsp_types::Position;

        let a = Range {
            start: Position {
                line: 1,
                character: 0,
            },
            end: Position {
                line: 2,
                character: 0,
            },
        };
        let b = Range {
            start: Position {
                line: 5,
                character: 0,
            },
            end: Position {
                line: 6,
                character: 0,
            },
        };
        assert!(!ranges_overlap(&a, &b));
    }

    #[test]
    fn diagnostic_includes_related_information() {
        use crate::{span::FileSpan, violation::Detection};

        let source = "if $x {\n    if $y {\n        foo\n    }\n}";
        let line_index = LineIndex::new(source);
        let file_uri: Uri = "file:///test.nu".parse().unwrap();

        let mut violation =
            Detection::from_file_span("Nested if can be collapsed", FileSpan::new(0, 39))
                .with_primary_label("outer if")
                .with_help("Combine with 'and'");
        violation
            .extra_labels
            .push((FileSpan::new(12, 35).into(), Some("inner if".to_string())));

        let mut violation = Violation::from_detected(violation, None);

        violation.lint_level = LintLevel::Warning;

        let diagnostic = violation_to_diagnostic(&violation, source, &line_index, &file_uri);

        assert!(diagnostic.message.contains("outer if"));
        assert!(diagnostic.message.contains("Combine with 'and'"));

        let related = diagnostic
            .related_information
            .expect("should have related_information");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].message, "inner if");
        assert_eq!(related[0].location.uri.as_str(), "file:///test.nu");
    }
}
