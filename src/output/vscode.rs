use std::collections::HashMap;

use serde::Serialize;

use super::{Summary, calculate_line_column, read_source_code};
use crate::{config::LintLevel, violation::Violation};

fn lint_level_to_severity(lint_level: LintLevel) -> u8 {
    match lint_level {
        LintLevel::Deny => 1,
        LintLevel::Warn => 2,
        LintLevel::Allow => unreachable!("Allow level violations should never be created"),
    }
}

#[must_use]
pub fn format_vscode_json(violations: &[Violation]) -> String {
    let mut diagnostics_by_file: HashMap<String, Vec<VsCodeDiagnostic>> = HashMap::new();

    for violation in violations {
        let file_path = violation
            .file
            .as_ref()
            .map_or_else(|| "unknown".to_string(), ToString::to_string);
        diagnostics_by_file
            .entry(file_path)
            .or_default()
            .push(violation_to_vscode_diagnostic(violation));
    }

    let summary = Summary::from_violations(violations);
    let output = VsCodeJsonOutput {
        diagnostics: diagnostics_by_file,
        summary,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

fn violation_to_vscode_diagnostic(violation: &Violation) -> VsCodeDiagnostic {
    let source_code = read_source_code(violation.file.as_ref());

    let (line_start, column_start) = calculate_line_column(&source_code, violation.span.start);
    let (line_end, column_end) = calculate_line_column(&source_code, violation.span.end);

    let line_start_zero = line_start.saturating_sub(1);
    let column_start_zero = column_start.saturating_sub(1);
    let line_end_zero = line_end.saturating_sub(1);
    let column_end_zero = column_end.saturating_sub(1);

    VsCodeDiagnostic {
        range: VsCodeRange {
            start: VsCodePosition {
                line: line_start_zero,
                character: column_start_zero,
            },
            end: VsCodePosition {
                line: line_end_zero,
                character: column_end_zero,
            },
        },
        severity: lint_level_to_severity(violation.lint_level),
        code: violation.rule_id.to_string(),
        source: "nu-lint".to_string(),
        message: violation.message.to_string(),
        related_information: violation.suggestion.as_ref().map(|suggestion| {
            vec![VsCodeRelatedInformation {
                location: VsCodeLocation {
                    uri: violation
                        .file
                        .as_ref()
                        .map_or_else(|| "unknown".to_string(), ToString::to_string),
                    range: VsCodeRange {
                        start: VsCodePosition {
                            line: line_start_zero,
                            character: column_start_zero,
                        },
                        end: VsCodePosition {
                            line: line_end_zero,
                            character: column_end_zero,
                        },
                    },
                },
                message: suggestion.to_string(),
            }]
        }),
        code_action: violation.fix.as_ref().map(|fix| VsCodeCodeAction {
            title: fix.description.to_string(),
            edits: fix
                .replacements
                .iter()
                .map(|r| {
                    let (r_line_start, r_col_start) =
                        calculate_line_column(&source_code, r.span.start);
                    let (r_line_end, r_col_end) = calculate_line_column(&source_code, r.span.end);
                    VsCodeTextEdit {
                        range: VsCodeRange {
                            start: VsCodePosition {
                                line: r_line_start.saturating_sub(1),
                                character: r_col_start.saturating_sub(1),
                            },
                            end: VsCodePosition {
                                line: r_line_end.saturating_sub(1),
                                character: r_col_end.saturating_sub(1),
                            },
                        },
                        new_text: r.new_text.to_string(),
                    }
                })
                .collect(),
        }),
    }
}

#[derive(Serialize)]
pub struct VsCodeJsonOutput {
    pub diagnostics: HashMap<String, Vec<VsCodeDiagnostic>>,
    pub summary: Summary,
}

#[derive(Serialize)]
pub struct VsCodeDiagnostic {
    pub range: VsCodeRange,
    pub severity: u8,
    pub code: String,
    pub source: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<Vec<VsCodeRelatedInformation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action: Option<VsCodeCodeAction>,
}

#[derive(Serialize)]
pub struct VsCodeRange {
    pub start: VsCodePosition,
    pub end: VsCodePosition,
}

#[derive(Serialize)]
pub struct VsCodePosition {
    pub line: usize,
    pub character: usize,
}

#[derive(Serialize)]
pub struct VsCodeRelatedInformation {
    pub location: VsCodeLocation,
    pub message: String,
}

#[derive(Serialize)]
pub struct VsCodeLocation {
    pub uri: String,
    pub range: VsCodeRange,
}

#[derive(Serialize)]
pub struct VsCodeCodeAction {
    pub title: String,
    pub edits: Vec<VsCodeTextEdit>,
}

#[derive(Serialize)]
pub struct VsCodeTextEdit {
    pub range: VsCodeRange,
    pub new_text: String,
}
