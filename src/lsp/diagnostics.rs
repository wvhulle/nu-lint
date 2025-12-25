use lsp_types::{
    CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location,
    NumberOrString, Range, Uri,
};

use super::line_index::LineIndex;
use crate::{LintLevel, violation::Violation};

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

#[cfg(test)]
mod tests {
    use lsp_types::{Position, Range};

    use super::ranges_overlap;
    use crate::violation::Detection;

    #[test]
    fn ranges_overlap_same_line() {
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
    fn diagnostic_includes_related_information() {
        use lsp_types::Uri;

        use super::{LineIndex, violation_to_diagnostic};
        use crate::{LintLevel, span::FileSpan, violation::Violation};

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
