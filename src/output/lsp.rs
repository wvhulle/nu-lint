use std::collections::HashMap;

use lsp_types::{
    CodeDescription, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location,
    NumberOrString, Position, Range, TextEdit, Uri,
};
use serde::Serialize;

use super::{Summary, calculate_line_column, read_source_code};
use crate::{config::LintLevel, violation::Violation};

fn lint_level_to_severity(lint_level: LintLevel) -> DiagnosticSeverity {
    match lint_level {
        LintLevel::Deny => DiagnosticSeverity::ERROR,
        LintLevel::Warn => DiagnosticSeverity::WARNING,
        LintLevel::Allow => unreachable!("Allow level violations should never be created"),
    }
}

/// Convert `usize` to `u32` for LSP, clamping to `u32::MAX` for very large
/// values
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

#[must_use]
pub fn format_lsp_json(violations: &[Violation]) -> String {
    let mut diagnostics_by_file: HashMap<String, Vec<Diagnostic>> = HashMap::new();

    for violation in violations {
        let file_path = violation
            .file
            .as_ref()
            .map_or_else(|| "unknown".to_string(), ToString::to_string);
        diagnostics_by_file
            .entry(file_path)
            .or_default()
            .push(violation_to_diagnostic(violation));
    }

    let summary = Summary::from_violations(violations);
    let output = LspJsonOutput {
        diagnostics: diagnostics_by_file,
        summary,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

fn violation_to_diagnostic(violation: &Violation) -> Diagnostic {
    let source_code = read_source_code(violation.file.as_ref());

    let (line_start, column_start) = calculate_line_column(&source_code, violation.span.start);
    let (line_end, column_end) = calculate_line_column(&source_code, violation.span.end);

    let line_start_zero = to_lsp_u32(line_start.saturating_sub(1));
    let column_start_zero = to_lsp_u32(column_start.saturating_sub(1));
    let line_end_zero = to_lsp_u32(line_end.saturating_sub(1));
    let column_end_zero = to_lsp_u32(column_end.saturating_sub(1));

    Diagnostic {
        range: Range {
            start: Position {
                line: line_start_zero,
                character: column_start_zero,
            },
            end: Position {
                line: line_end_zero,
                character: column_end_zero,
            },
        },
        severity: Some(lint_level_to_severity(violation.lint_level)),
        code: Some(NumberOrString::String(
            violation
                .rule_id
                .as_deref()
                .unwrap_or("unknown")
                .to_string(),
        )),
        code_description: violation
            .doc_url
            .and_then(|url| url.parse::<Uri>().ok().map(|href| CodeDescription { href })),
        source: Some("nu-lint".to_string()),
        message: violation.message.to_string(),
        tags: None,
        related_information: violation.help.as_ref().map(|suggestion| {
            let file_uri = violation
                .file
                .as_ref()
                .and_then(|f| f.parse::<Uri>().ok())
                .unwrap_or_else(|| "file:///unknown".parse().expect("valid URI"));
            vec![DiagnosticRelatedInformation {
                location: Location {
                    uri: file_uri,
                    range: Range {
                        start: Position {
                            line: line_start_zero,
                            character: column_start_zero,
                        },
                        end: Position {
                            line: line_end_zero,
                            character: column_end_zero,
                        },
                    },
                },
                message: suggestion.to_string(),
            }]
        }),
        data: violation.fix.as_ref().map(|fix| {
            serde_json::to_value(QuickFix {
                title: fix.explanation.to_string(),
                edits: fix
                    .replacements
                    .iter()
                    .map(|r| {
                        let (r_line_start, r_col_start) =
                            calculate_line_column(&source_code, r.span.start);
                        let (r_line_end, r_col_end) =
                            calculate_line_column(&source_code, r.span.end);
                        TextEdit {
                            range: Range {
                                start: Position {
                                    line: to_lsp_u32(r_line_start.saturating_sub(1)),
                                    character: to_lsp_u32(r_col_start.saturating_sub(1)),
                                },
                                end: Position {
                                    line: to_lsp_u32(r_line_end.saturating_sub(1)),
                                    character: to_lsp_u32(r_col_end.saturating_sub(1)),
                                },
                            },
                            new_text: r.replacement_text.to_string(),
                        }
                    })
                    .collect(),
            })
            .expect("QuickFix serializable")
        }),
    }
}

/// Output format containing diagnostics grouped by file URI
#[derive(Serialize)]
pub struct LspJsonOutput {
    pub diagnostics: HashMap<String, Vec<Diagnostic>>,
    pub summary: Summary,
}

/// Quick fix data stored in diagnostic's data field for code action support
#[derive(Serialize)]
pub struct QuickFix {
    /// Human-readable title for the fix
    pub title: String,
    /// Text edits to apply
    pub edits: Vec<TextEdit>,
}
