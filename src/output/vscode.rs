//! VS Code JSON output format (deprecated, use `lsp` format instead)
//!
//! This module is kept for backwards compatibility with existing VS Code
//! extensions. New integrations should use the `lsp` module which follows the
//! standard LSP 3.17 spec.

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

/// Format violations as VS Code JSON
#[must_use]
pub fn format_vscode_json(violations: &[Violation]) -> String {
    let mut diagnostics_by_file: HashMap<String, Vec<VsCodeDiagnostic>> = HashMap::new();

    for violation in violations {
        let file_path = violation
            .file
            .as_ref()
            .map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);
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

    let (line_start, column_start) = calculate_line_column(&source_code, violation.span.start());
    let (line_end, column_end) = calculate_line_column(&source_code, violation.span.end());

    let line_start_zero = line_start.saturating_sub(1);
    let column_start_zero = column_start.saturating_sub(1);
    let line_end_zero = line_end.saturating_sub(1);
    let column_end_zero = column_end.saturating_sub(1);

    let file_uri = violation
        .file
        .as_ref()
        .map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);

    let related_information = build_related_information(violation, &source_code, &file_uri);

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
        code: violation
            .rule_id
            .as_deref()
            .unwrap_or("unknown")
            .to_string(),
        source: "nu-lint".to_string(),
        message: build_message(violation),
        code_description: violation.doc_url.map(|url| VsCodeCodeDescription {
            href: url.to_string(),
        }),
        related_information: if related_information.is_empty() {
            None
        } else {
            Some(related_information)
        },
        code_action: violation.fix.as_ref().map(|fix| VsCodeCodeAction {
            title: fix.explanation.to_string(),
            edits: fix
                .replacements
                .iter()
                .enumerate()
                .map(|(idx, r)| {
                    let (r_line_start, r_col_start) =
                        calculate_line_column(&source_code, r.span.start());
                    let (r_line_end, r_col_end) = calculate_line_column(&source_code, r.span.end());
                    let description = if fix.replacements.len() == 1 {
                        Some(fix.explanation.to_string())
                    } else {
                        Some(format!("{} (edit {})", fix.explanation, idx + 1))
                    };
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
                        replacement_text: r.replacement_text.to_string(),
                        description,
                    }
                })
                .collect(),
        }),
    }
}

fn build_message(violation: &Violation) -> String {
    violation.primary_label.as_ref().map_or_else(
        || violation.message.to_string(),
        |label| format!("{}: {label}", violation.message),
    )
}

fn build_related_information(
    violation: &Violation,
    source_code: &str,
    file_uri: &str,
) -> Vec<VsCodeRelatedInformation> {
    let mut info = Vec::new();

    for (span, label) in &violation.extra_labels {
        let label_text = label.as_deref().unwrap_or_default();
        if label_text.is_empty() {
            continue;
        }

        let (line_start, col_start) = calculate_line_column(source_code, span.start());
        let (line_end, col_end) = calculate_line_column(source_code, span.end());

        info.push(VsCodeRelatedInformation {
            location: VsCodeLocation {
                uri: file_uri.to_string(),
                range: VsCodeRange {
                    start: VsCodePosition {
                        line: line_start.saturating_sub(1),
                        character: col_start.saturating_sub(1),
                    },
                    end: VsCodePosition {
                        line: line_end.saturating_sub(1),
                        character: col_end.saturating_sub(1),
                    },
                },
            },
            message: label_text.to_string(),
        });
    }

    if let Some(help) = &violation.help {
        let (line_start, col_start) = calculate_line_column(source_code, violation.span.start());
        let (line_end, col_end) = calculate_line_column(source_code, violation.span.end());

        info.push(VsCodeRelatedInformation {
            location: VsCodeLocation {
                uri: file_uri.to_string(),
                range: VsCodeRange {
                    start: VsCodePosition {
                        line: line_start.saturating_sub(1),
                        character: col_start.saturating_sub(1),
                    },
                    end: VsCodePosition {
                        line: line_end.saturating_sub(1),
                        character: col_end.saturating_sub(1),
                    },
                },
            },
            message: format!("Help: {help}"),
        });
    }

    info
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
    #[serde(skip_serializing_if = "Option::is_none", rename = "codeDescription")]
    pub code_description: Option<VsCodeCodeDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<Vec<VsCodeRelatedInformation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action: Option<VsCodeCodeAction>,
}

#[derive(Serialize)]
pub struct VsCodeCodeDescription {
    pub href: String,
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
    pub replacement_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
