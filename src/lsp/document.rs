use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeDescription, Diagnostic,
    DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Position, Range,
    TextEdit, Uri, WorkspaceEdit,
};

use crate::{Config, LintEngine, LintLevel, violation::Violation};

#[allow(
    clippy::cast_possible_truncation,
    reason = "LSP uses u32, files larger than 4GB lines/columns are unrealistic"
)]
#[must_use]
const fn to_lsp_u32(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

/// Pre-computed line offset index for efficient byte offset to line/column
/// conversion. Uses binary search for O(log n) lookups instead of O(n)
/// iteration.
pub struct LineIndex {
    /// Byte offsets where each line starts. `line_offsets[0]` = 0 (first line
    /// starts at byte 0).
    line_offsets: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_offsets = vec![0];
        for (pos, ch) in source.char_indices() {
            if ch == '\n' {
                line_offsets.push(pos + 1);
            }
        }
        Self { line_offsets }
    }

    /// Convert a byte offset to LSP Position (0-indexed line and column).
    pub fn offset_to_position(&self, offset: usize, source: &str) -> Position {
        let line = self
            .line_offsets
            .partition_point(|&line_start| line_start <= offset)
            .saturating_sub(1);

        let line_start = self.line_offsets.get(line).copied().unwrap_or(0);
        let end_offset = offset.min(source.len());
        let column = source
            .get(line_start..end_offset)
            .map_or(0, |s| s.chars().count());

        Position {
            line: to_lsp_u32(line),
            character: to_lsp_u32(column),
        }
    }

    pub fn span_to_range(&self, source: &str, start: usize, end: usize) -> Range {
        Range {
            start: self.offset_to_position(start, source),
            end: self.offset_to_position(end, source),
        }
    }
}

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

        let mut diagnostics = vec![];

        for violation in &violations {
            diagnostics.push(violation_to_diagnostic(
                violation,
                content,
                &line_index,
                uri,
            ));

            diagnostics.extend(create_extra_label_diagnostics(
                violation,
                content,
                &line_index,
                uri,
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

    pub fn has_document(&self, uri: &Uri) -> bool {
        self.documents.contains_key(uri)
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
    // Use only the short message for LSP diagnostics.
    // The primary_label is redundant for inline display since the underline
    // already indicates the location. Appending it creates overly long messages
    // like "Unnecessary '^' prefix: redundant prefix" which are hard to read
    // in editors that show diagnostics inline (Helix, reedline, etc.)
    violation.message.to_string()
}

fn build_related_information(
    violation: &Violation,
    source: &str,
    line_index: &LineIndex,
    file_uri: &Uri,
) -> Vec<DiagnosticRelatedInformation> {
    let mut result = Vec::new();
    let mut seen_ranges = HashSet::new();

    // Add primary_label as the first related information entry.
    // This keeps the main diagnostic message short while preserving
    // the detailed context for IDEs that display related information.
    if let Some(ref label) = violation.primary_label {
        if !label.is_empty() {
            let file_span = violation.file_span();
            let range = line_index.span_to_range(source, file_span.start, file_span.end);
            let range_key = (
                range.start.line,
                range.start.character,
                range.end.line,
                range.end.character,
            );
            seen_ranges.insert(range_key);
            result.push(DiagnosticRelatedInformation {
                location: Location {
                    uri: file_uri.clone(),
                    range,
                },
                message: label.to_string(),
            });
        }
    }

    // Add extra_labels as additional related information
    for (span, label) in &violation.extra_labels {
        let Some(label_text) = label.as_deref() else {
            continue;
        };
        if label_text.is_empty() {
            continue;
        }

        let file_span = span.file_span();
        let range = line_index.span_to_range(source, file_span.start, file_span.end);

        let range_key = (
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character,
        );
        if !seen_ranges.insert(range_key) {
            continue;
        }

        result.push(DiagnosticRelatedInformation {
            location: Location {
                uri: file_uri.clone(),
                range,
            },
            message: label_text.to_string(),
        });
    }

    result
}

fn create_extra_label_diagnostics(
    violation: &Violation,
    source: &str,
    line_index: &LineIndex,
    _file_uri: &Uri,
) -> Vec<Diagnostic> {
    let primary_span = violation.file_span();
    let primary_range = line_index.span_to_range(source, primary_span.start, primary_span.end);

    let mut seen_ranges = HashSet::new();

    let result: Vec<Diagnostic> = violation
        .extra_labels
        .iter()
        .filter_map(|(span, label)| {
            let file_span = span.file_span();
            let range = line_index.span_to_range(source, file_span.start, file_span.end);

            log::debug!(
                "Processing extra_label: span={:?}, range=({},{}) to ({},{}), label={:?}",
                file_span,
                range.start.line,
                range.start.character,
                range.end.line,
                range.end.character,
                label
            );

            if range.start == primary_range.start && range.end == primary_range.end {
                log::debug!("  -> Filtered: same as primary range");
                return None;
            }

            let range_key = (
                range.start.line,
                range.start.character,
                range.end.line,
                range.end.character,
            );
            if !seen_ranges.insert(range_key) {
                log::debug!("  -> Filtered: duplicate range");
                return None;
            }

            let message = label
                .as_deref()
                .map_or_else(|| String::from("Related location"), ToString::to_string);

            log::debug!("  -> Creating HINT diagnostic with message: {message}");

            Some(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::HINT),
                code: violation
                    .rule_id
                    .as_deref()
                    .map(|id| NumberOrString::String(id.to_string())),
                code_description: None,
                source: Some(String::from("nu-lint")),
                message,
                related_information: None,
                tags: None,
                data: None,
            })
        })
        .collect();

    log::debug!(
        "Created {} HINT diagnostics from {} extra_labels",
        result.len(),
        violation.extra_labels.len()
    );

    result
}

#[must_use]
pub const fn ranges_overlap(a: &Range, b: &Range) -> bool {
    a.start.line <= b.end.line && b.start.line <= a.end.line
}

/// Check if a language_id indicates nushell.
#[must_use]
pub const fn is_nushell_language_id(language_id: &str) -> bool {
    language_id.eq_ignore_ascii_case("nushell") || language_id.eq_ignore_ascii_case("nu")
}

/// Check if a URI looks like a nushell file based on extension or scheme.
/// This is a fallback for clients that don't set language_id properly.
#[must_use]
pub fn is_nushell_file(uri: &Uri) -> bool {
    // Accept repl:// URIs for REPL integration
    if uri.scheme().is_some_and(|s| s.as_str() == "repl") {
        return true;
    }
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
    fn is_nushell_file_wrong_extension() {
        let uri: Uri = "file:///path/to/script.rs".parse().unwrap();
        assert!(!is_nushell_file(&uri));
    }

    #[test]
    fn is_nushell_file_repl_uri() {
        let uri: Uri = "repl:/session/repl".parse().unwrap();
        assert!(is_nushell_file(&uri));
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
                .with_primary_label("outer if");
        violation
            .extra_labels
            .push((FileSpan::new(12, 35).into(), Some("inner if".to_string())));

        let mut violation = Violation::from_detected(violation, None, Some("Combine with 'and'"));

        violation.lint_level = LintLevel::Warning;

        let diagnostic = violation_to_diagnostic(&violation, source, &line_index, &file_uri);

        // Message should be short (without primary_label appended)
        assert_eq!(diagnostic.message, "Nested if can be collapsed");

        // primary_label ("outer if") should now be in related_information
        let related = diagnostic
            .related_information
            .expect("should have related_information");
        assert_eq!(related.len(), 2); // primary_label + extra_label
        assert_eq!(related[0].message, "outer if"); // primary_label first
        assert_eq!(related[1].message, "inner if"); // extra_label second
        assert_eq!(related[0].location.uri.as_str(), "file:///test.nu");
    }

    #[test]
    fn line_index_multiple_lines() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);
        assert_eq!(index.line_offsets, vec![0, 6, 12]);
    }

    #[test]
    fn offset_to_position_multiple_lines() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);

        let pos = index.offset_to_position(12, source);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);

        let pos = index.offset_to_position(14, source);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 2);
    }

    #[test]
    fn offset_to_position_beyond_end() {
        let source = "hello";
        let index = LineIndex::new(source);
        let pos = index.offset_to_position(100, source);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn span_to_range_single_line() {
        let source = "let x = 5";
        let index = LineIndex::new(source);
        let range = index.span_to_range(source, 4, 5);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 4);
        assert_eq!(range.end.line, 0);
        assert_eq!(range.end.character, 5);
    }

    #[test]
    fn span_to_range_multiline() {
        let source = "def foo [] {\n    bar\n}";
        let index = LineIndex::new(source);
        let range = index.span_to_range(source, 0, 22);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 0);
        assert_eq!(range.end.line, 2);
    }

    #[test]
    fn extra_labels_create_hint_diagnostics() {
        use crate::{span::FileSpan, violation::Detection};

        let source = "def foo [] {\n    [1, 2, 3]\n}";
        let line_index = LineIndex::new(source);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        let mut violation =
            Detection::from_file_span("Function missing output type", FileSpan::new(0, 9))
                .with_primary_label("add output type");

        violation.extra_labels.push((
            FileSpan::new(17, 26).into(),
            Some("returned here".to_string()),
        ));

        let mut violation = Violation::from_detected(violation, None, None);
        violation.rule_id = Some("missing_output_type".into());
        violation.lint_level = LintLevel::Warning;

        let diagnostics = [violation_to_diagnostic(
            &violation,
            source,
            &line_index,
            &uri,
        )];
        let extra_diagnostics =
            create_extra_label_diagnostics(&violation, source, &line_index, &uri);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(extra_diagnostics.len(), 1);

        let hint_diag = &extra_diagnostics[0];
        assert_eq!(hint_diag.severity, Some(DiagnosticSeverity::HINT));
        assert_eq!(hint_diag.message, "returned here");
        assert_eq!(hint_diag.range.start.line, 1);
    }

    #[test]
    fn extra_labels_without_text_create_hints() {
        use crate::{span::FileSpan, violation::Detection};

        let source = "def foo [] {\n    bar\n}";
        let line_index = LineIndex::new(source);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        let mut violation = Detection::from_file_span("Test violation", FileSpan::new(0, 9));
        violation
            .extra_labels
            .push((FileSpan::new(17, 20).into(), None));

        let violation = Violation::from_detected(violation, None, None);

        let extra_diagnostics =
            create_extra_label_diagnostics(&violation, source, &line_index, &uri);

        assert_eq!(extra_diagnostics.len(), 1);
        assert_eq!(extra_diagnostics[0].message, "Related location");
        assert_eq!(
            extra_diagnostics[0].severity,
            Some(DiagnosticSeverity::HINT)
        );
    }

    #[test]
    fn multiple_extra_labels_create_multiple_hints() {
        use crate::{span::FileSpan, violation::Detection};

        let source = "if $x {\n    if $y {\n        foo\n    }\n}";
        let line_index = LineIndex::new(source);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        let mut violation =
            Detection::from_file_span("Nested if can be collapsed", FileSpan::new(0, 7));
        violation
            .extra_labels
            .push((FileSpan::new(12, 19).into(), Some("inner if".to_string())));
        violation
            .extra_labels
            .push((FileSpan::new(28, 31).into(), Some("inner body".to_string())));

        let violation = Violation::from_detected(violation, None, None);

        let extra_diagnostics =
            create_extra_label_diagnostics(&violation, source, &line_index, &uri);

        assert_eq!(extra_diagnostics.len(), 2);
        assert_eq!(extra_diagnostics[0].message, "inner if");
        assert_eq!(extra_diagnostics[1].message, "inner body");
    }

    #[test]
    fn lint_document_returns_main_and_hint_diagnostics() {
        let config = Config::default();
        let mut state = ServerState::new(config);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        let source = "def foo [] {\n    [1, 2, 3]\n}";

        let diagnostics = state.lint_document(&uri, source);

        for diag in &diagnostics {
            println!(
                "Diagnostic: severity={:?}, message={}",
                diag.severity, diag.message
            );
        }
    }

    #[test]
    fn extra_labels_with_same_span_as_primary_are_filtered() {
        use crate::{span::FileSpan, violation::Detection};

        let source = "let x = (";
        let line_index = LineIndex::new(source);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        let primary_span = FileSpan::new(0, 9);
        let mut violation = Detection::from_file_span("Unclosed parenthesis", primary_span)
            .with_primary_label("expected closing )");

        violation
            .extra_labels
            .push((primary_span.into(), Some("here".to_string())));

        violation.extra_labels.push((
            FileSpan::new(5, 6).into(),
            Some("different location".to_string()),
        ));

        let violation = Violation::from_detected(violation, None, None);

        let extra_diagnostics =
            create_extra_label_diagnostics(&violation, source, &line_index, &uri);

        assert_eq!(extra_diagnostics.len(), 1);
        assert_eq!(extra_diagnostics[0].message, "different location");
    }

    #[test]
    fn duplicate_extra_label_ranges_are_filtered() {
        use crate::{span::FileSpan, violation::Detection};

        let source = "mut result = []\n$result = $result ++ [1]";
        let line_index = LineIndex::new(source);
        let uri: Uri = "file:///test.nu".parse().unwrap();

        let mut violation =
            Detection::from_file_span("Capture of mutable variable", FileSpan::new(0, 15));

        violation.extra_labels.push((
            FileSpan::new(16, 23).into(),
            Some("Capture of mutable variable".to_string()),
        ));
        violation.extra_labels.push((
            FileSpan::new(16, 23).into(),
            Some("Capture of mutable variable".to_string()),
        ));
        violation.extra_labels.push((
            FileSpan::new(16, 23).into(),
            Some("Capture of mutable variable".to_string()),
        ));

        let violation = Violation::from_detected(violation, None, None);

        let extra_diagnostics =
            create_extra_label_diagnostics(&violation, source, &line_index, &uri);

        assert_eq!(extra_diagnostics.len(), 1);
        assert_eq!(extra_diagnostics[0].message, "Capture of mutable variable");
    }
}
