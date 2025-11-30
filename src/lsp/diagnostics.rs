use lsp_types::{CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Range};

use super::line_index::LineIndex;
use crate::{LintLevel, violation::Violation};

const fn lint_level_to_severity(level: LintLevel) -> DiagnosticSeverity {
    match level {
        LintLevel::Deny => DiagnosticSeverity::ERROR,
        LintLevel::Warn => DiagnosticSeverity::WARNING,
        LintLevel::Allow => DiagnosticSeverity::HINT,
    }
}

pub fn violation_to_diagnostic(
    violation: &Violation,
    source: &str,
    line_index: &LineIndex,
) -> Diagnostic {
    let message = violation.help.as_ref().map_or_else(
        || violation.message.to_string(),
        |help| format!("{}\n\nHelp: {help}", violation.message),
    );

    Diagnostic {
        range: line_index.span_to_range(source, violation.span.start, violation.span.end),
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
        related_information: None,
        tags: None,
        data: None,
    }
}

#[must_use]
pub const fn ranges_overlap(a: &Range, b: &Range) -> bool {
    a.start.line <= b.end.line && b.start.line <= a.end.line
}

#[cfg(test)]
mod tests {
    use lsp_types::{Position, Range};

    use super::ranges_overlap;

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
}
