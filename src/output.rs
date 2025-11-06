use core::{error::Error, iter};
use std::{fmt, fs};

use miette::{Diagnostic, LabeledSpan, Report, SourceCode};
use serde::Serialize;

use crate::{
    Fix,
    violation::{Severity, Violation},
};

/// Format violations as human-readable text
pub fn format_text(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return String::from("No violations found!");
    }

    let summary = Summary::from_violations(violations);
    let header = format!("Found {}\n", summary.format_compact());

    let violations_output: String = violations
        .iter()
        .enumerate()
        .map(|(idx, violation)| format_violation_text(violation, idx < violations.len() - 1))
        .collect();

    let footer = format!("\n{}", summary.format_compact());

    format!("{header}{violations_output}{footer}")
}

/// Format a single violation as text
fn format_violation_text(violation: &Violation, add_separator: bool) -> String {
    let source_code = violation
        .file
        .as_ref()
        .and_then(|path| fs::read_to_string(path.as_ref()).ok())
        .unwrap_or_default();

    let (line, column) = calculate_line_column(&source_code, violation.span.start);
    let (end_line, end_column) = calculate_line_column(&source_code, violation.span.end);

    let header = violation.file.as_ref().map_or(String::new(), |file_path| {
        format!("\n\x1b[1;4m{file_path}:{line}:{column}\x1b[0m\n")
    });

    let diagnostic = ViolationDiagnostic {
        violation: violation.clone(),
        source_code: source_code.clone(),
        line,
        column,
        end_line,
        end_column,
    };

    let report = format!("{:?}", Report::new(diagnostic));

    let fix_info = violation
        .fix
        .as_ref()
        .map(|fix| format_fix_info(fix, &source_code))
        .unwrap_or_default();

    let separator = if add_separator {
        format!("\n\n{}\n", "─".repeat(80))
    } else {
        String::new()
    };

    format!("{header}{report}\n{fix_info}{separator}")
}

/// Format violations as JSON
pub fn format_json(violations: &[Violation]) -> String {
    let json_violations: Vec<JsonViolation> = violations.iter().map(violation_to_json).collect();

    let summary = Summary::from_violations(violations);
    let output = JsonOutput {
        violations: json_violations,
        summary,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

/// Calculate line and column number from byte offset in source
/// Returns (line, column) as 1-indexed values
fn calculate_line_column(source: &str, offset: usize) -> (usize, usize) {
    source
        .char_indices()
        .take_while(|(pos, _)| *pos < offset)
        .fold((1, 1), |(line, column), (_, ch)| {
            if ch == '\n' {
                (line + 1, 1)
            } else {
                (line, column + 1)
            }
        })
}

/// Format fix information for text output
fn format_fix_info(fix: &Fix, source_code: &str) -> String {
    let header = format!("\n  \x1b[36mℹ Available fix:\x1b[0m {}", fix.description);

    if fix.replacements.is_empty() {
        return header;
    }

    let replacements = fix
        .replacements
        .iter()
        .map(|replacement| {
            let (start_line, start_col) =
                calculate_line_column(source_code, replacement.span.start);
            let (end_line, end_col) = calculate_line_column(source_code, replacement.span.end);
            format!(
                "    • {}:{}-{}:{} → {}",
                start_line, start_col, end_line, end_col, replacement.new_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{header}\n  \x1b[2mReplacements:\x1b[0m\n{replacements}")
}

/// Convert a violation to JSON format
fn violation_to_json(violation: &Violation) -> JsonViolation {
    let source_code = violation
        .file
        .as_ref()
        .and_then(|path| fs::read_to_string(path.as_ref()).ok())
        .unwrap_or_default();

    let (line_start, column_start) = calculate_line_column(&source_code, violation.span.start);
    let (line_end, column_end) = calculate_line_column(&source_code, violation.span.end);

    JsonViolation {
        rule_id: violation.rule_id.to_string(),
        severity: violation.severity.to_string(),
        message: violation.message.to_string(),
        file: violation.file.as_ref().map(ToString::to_string),
        line_start,
        line_end,
        column_start,
        column_end,
        offset_start: violation.span.start,
        offset_end: violation.span.end,
        suggestion: violation.suggestion.as_ref().map(ToString::to_string),
        fix: violation.fix.as_ref().map(fix_to_json),
    }
}

/// Convert a fix to JSON format
fn fix_to_json(fix: &Fix) -> JsonFix {
    JsonFix {
        description: fix.description.to_string(),
        replacements: fix
            .replacements
            .iter()
            .map(|r| JsonReplacement {
                offset_start: r.span.start,
                offset_end: r.span.end,
                new_text: r.new_text.to_string(),
            })
            .collect(),
    }
}

#[derive(Debug, Clone)]
struct ViolationDiagnostic {
    violation: Violation,
    source_code: String,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
}

impl fmt::Display for ViolationDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.violation.message)
    }
}

impl Error for ViolationDiagnostic {}

impl Diagnostic for ViolationDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(format!(
            "{}({})",
            self.violation.severity, self.violation.rule_id
        )))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(match self.violation.severity {
            Severity::Error => miette::Severity::Error,
            Severity::Warning => miette::Severity::Warning,
            Severity::Info => miette::Severity::Advice,
        })
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.violation
            .suggestion
            .as_deref()
            .map(|s| Box::new(s) as Box<dyn fmt::Display>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let span = self.violation.to_source_span();
        let label_text = if self.line == self.end_line {
            format!("{} [{}:{}]", self.violation.message, self.line, self.column)
        } else {
            format!(
                "{} [{}:{} - {}:{}]",
                self.violation.message, self.line, self.column, self.end_line, self.end_column
            )
        };

        Some(Box::new(iter::once(LabeledSpan::new(
            Some(label_text),
            span.offset(),
            span.len(),
        ))))
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source_code as &dyn SourceCode)
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
    pub severity: String,
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

#[derive(Serialize)]
pub struct Summary {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub files_checked: usize,
}

impl Summary {
    #[must_use]
    pub fn from_violations(violations: &[Violation]) -> Self {
        let (errors, warnings, info) = violations.iter().fold(
            (0, 0, 0),
            |(errors, warnings, info), violation| match violation.severity {
                Severity::Error => (errors + 1, warnings, info),
                Severity::Warning => (errors, warnings + 1, info),
                Severity::Info => (errors, warnings, info + 1),
            },
        );

        Self {
            errors,
            warnings,
            info,
            files_checked: 1,
        }
    }

    /// Format summary showing only non-zero severity counts
    #[must_use]
    pub fn format_compact(&self) -> String {
        let parts: Vec<String> = [
            (self.errors > 0).then(|| format!("{} error(s)", self.errors)),
            (self.warnings > 0).then(|| format!("{} warning(s)", self.warnings)),
            (self.info > 0).then(|| format!("{} info", self.info)),
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            String::from("0 violations")
        } else {
            parts.join(", ")
        }
    }
}
