use std::{collections::HashSet, path::Path};

use lsp_types::{
    CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location,
    NumberOrString, Position, Range, Uri,
};

use crate::{LintLevel, violation::Violation};

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

/// Byte offset to line/column index using binary search.
pub struct LineIndex {
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

    /// Get the line number for a byte offset
    pub fn offset_to_line(&self, offset: usize) -> usize {
        self.line_offsets
            .partition_point(|&start| start <= offset)
            .saturating_sub(1)
    }

    /// Get the byte offset of a line's start
    pub fn line_start(&self, line: usize) -> usize {
        self.line_offsets.get(line).copied().unwrap_or(0)
    }

    /// Get the content of a specific line (without trailing newline)
    pub fn line_content<'a>(&self, source: &'a str, line: usize) -> &'a str {
        let start = self.line_start(line);
        let end = self
            .line_offsets
            .get(line + 1)
            .copied()
            .unwrap_or(source.len());
        source.get(start..end).unwrap_or("").trim_end_matches('\n')
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
    let related_info = collect_related_info(violation, source, line_index, file_uri);
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
        message: violation.message.to_string(),
        related_information: if related_info.is_empty() {
            None
        } else {
            Some(related_info)
        },
        tags: if violation.diagnostic_tags.is_empty() {
            None
        } else {
            Some(violation.diagnostic_tags.clone())
        },
        data: None,
    }
}

fn collect_related_info(
    violation: &Violation,
    source: &str,
    line_index: &LineIndex,
    file_uri: &Uri,
) -> Vec<DiagnosticRelatedInformation> {
    let primary = violation
        .primary_label
        .as_ref()
        .filter(|l| !l.is_empty())
        .map(|label| (violation.file_span(), label.as_ref()));

    let extras = violation.extra_labels.iter().filter_map(|(span, label)| {
        label
            .as_deref()
            .filter(|l| !l.is_empty())
            .map(|l| (span.file_span(), l))
    });

    primary
        .into_iter()
        .chain(extras)
        .scan(HashSet::new(), |seen, (file_span, label)| {
            let range = line_index.span_to_range(source, file_span.start, file_span.end);
            let key = (
                range.start.line,
                range.start.character,
                range.end.line,
                range.end.character,
            );
            Some(seen.insert(key).then(|| DiagnosticRelatedInformation {
                location: Location {
                    uri: file_uri.clone(),
                    range,
                },
                message: label.to_string(),
            }))
        })
        .flatten()
        .collect()
}

pub fn extra_labels_to_hint_diagnostics(
    violation: &Violation,
    source: &str,
    line_index: &LineIndex,
) -> Vec<Diagnostic> {
    let primary_span = violation.file_span();
    let primary_range = line_index.span_to_range(source, primary_span.start, primary_span.end);
    let mut seen_ranges = HashSet::new();

    violation
        .extra_labels
        .iter()
        .filter_map(|(span, label)| {
            let file_span = span.file_span();
            let range = line_index.span_to_range(source, file_span.start, file_span.end);

            if range == primary_range {
                return None;
            }

            let range_key = (
                range.start.line,
                range.start.character,
                range.end.line,
                range.end.character,
            );
            if !seen_ranges.insert(range_key) {
                return None;
            }

            let message = label
                .as_deref()
                .map_or_else(|| String::from("Related location"), ToString::to_string);

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
        .collect()
}

#[must_use]
pub const fn ranges_overlap(a: &Range, b: &Range) -> bool {
    a.start.line <= b.end.line && b.start.line <= a.end.line
}

#[must_use]
pub const fn is_nushell_language_id(language_id: &str) -> bool {
    language_id.eq_ignore_ascii_case("nushell") || language_id.eq_ignore_ascii_case("nu")
}

#[must_use]
pub fn is_nushell_uri(uri: &Uri) -> bool {
    uri.scheme().is_some_and(|s| s.as_str() == "repl")
        || Path::new(uri.path().as_str())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nu"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{span::FileSpan, violation::Detection};

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
    fn diagnostic_includes_related_information() {
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

        assert_eq!(diagnostic.message, "Nested if can be collapsed");
        let related = diagnostic
            .related_information
            .expect("should have related_information");
        assert_eq!(related.len(), 2);
        assert_eq!(related[0].message, "outer if");
        assert_eq!(related[1].message, "inner if");
    }

    #[test]
    fn ranges_overlap_different_lines() {
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
    fn extra_labels_with_same_span_as_primary_are_filtered() {
        let source = "let x = (";
        let line_index = LineIndex::new(source);

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

        let hints = extra_labels_to_hint_diagnostics(&violation, source, &line_index);

        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].message, "different location");
    }

    #[test]
    fn duplicate_extra_label_ranges_are_filtered() {
        let source = "mut result = []\n$result = $result ++ [1]";
        let line_index = LineIndex::new(source);

        let mut violation =
            Detection::from_file_span("Capture of mutable variable", FileSpan::new(0, 15));

        for _ in 0..3 {
            violation.extra_labels.push((
                FileSpan::new(16, 23).into(),
                Some("Capture of mutable variable".to_string()),
            ));
        }

        let violation = Violation::from_detected(violation, None, None);
        let hints = extra_labels_to_hint_diagnostics(&violation, source, &line_index);

        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].message, "Capture of mutable variable");
    }

    #[test]
    fn extra_labels_create_hint_diagnostics() {
        let source = "def foo [] {\n    [1, 2, 3]\n}";
        let line_index = LineIndex::new(source);

        let mut violation =
            Detection::from_file_span("Function missing output type", FileSpan::new(0, 9))
                .with_primary_label("add output type");

        violation.extra_labels.push((
            FileSpan::new(17, 26).into(),
            Some("returned here".to_string()),
        ));

        let violation = Violation::from_detected(violation, None, None);
        let hints = extra_labels_to_hint_diagnostics(&violation, source, &line_index);

        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].severity, Some(DiagnosticSeverity::HINT));
        assert_eq!(hints[0].message, "returned here");
        assert_eq!(hints[0].range.start.line, 1);
    }
}
