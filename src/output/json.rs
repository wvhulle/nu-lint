use serde::Serialize;

use super::{Summary, calculate_line_column, read_source_code};
use crate::{Fix, violation::Violation};

pub fn format_json(violations: &[Violation]) -> String {
    let json_violations: Vec<JsonViolation> = violations.iter().map(violation_to_json).collect();

    let summary = Summary::from_violations(violations);
    let output = JsonOutput {
        violations: json_violations,
        summary,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

fn violation_to_json(violation: &Violation) -> JsonViolation {
    let source_code = read_source_code(violation.file.as_ref());

    let (line_start, column_start) = calculate_line_column(&source_code, violation.span.start);
    let (line_end, column_end) = calculate_line_column(&source_code, violation.span.end);

    JsonViolation {
        rule_id: violation.rule_id.as_deref().unwrap_or("unknown").to_string(),
        lint_level: violation.lint_level.to_string(),
        message: violation.message.to_string(),
        file: violation.file.as_ref().map(ToString::to_string),
        line_start,
        line_end,
        column_start,
        column_end,
        offset_start: violation.span.start,
        offset_end: violation.span.end,
        suggestion: violation.help.as_ref().map(ToString::to_string),
        fix: violation.fix.as_ref().map(fix_to_json),
        doc_url: violation.doc_url.map(ToString::to_string),
    }
}

fn fix_to_json(fix: &Fix) -> JsonFix {
    JsonFix {
        description: fix.explanation.to_string(),
        replacements: fix
            .replacements
            .iter()
            .map(|r| JsonReplacement {
                offset_start: r.span.start,
                offset_end: r.span.end,
                new_text: r.replacement_text.to_string(),
            })
            .collect(),
    }
}

#[derive(Serialize)]
pub struct JsonOutput {
    pub violations: Vec<JsonViolation>,
    pub summary: Summary,
}

#[derive(Serialize)]
pub struct JsonViolation {
    pub rule_id: String,
    pub lint_level: String,
    pub message: String,
    pub file: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub offset_start: usize,
    pub offset_end: usize,
    pub suggestion: Option<String>,
    pub fix: Option<JsonFix>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_url: Option<String>,
}

#[derive(Serialize)]
pub struct JsonFix {
    pub description: String,
    pub replacements: Vec<JsonReplacement>,
}

#[derive(Serialize)]
pub struct JsonReplacement {
    pub offset_start: usize,
    pub offset_end: usize,
    pub new_text: String,
}
