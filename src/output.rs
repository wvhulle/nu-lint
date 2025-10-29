use std::{fmt, fmt::Write};

use miette::{Diagnostic, LabeledSpan, Report, SourceCode};
use serde::Serialize;

use crate::lint::{Severity, Violation};
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Github,
}

pub trait OutputFormatter {
    fn format(&self, violations: &[Violation], source: &str) -> String;
}

#[derive(Debug, Default)]
pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn format(&self, violations: &[Violation], _source: &str) -> String {
        if violations.is_empty() {
            return String::from("No violations found!");
        }

        let mut output = String::new();

        // Show summary at the beginning
        let summary = Summary::from_violations(violations);
        let _ = writeln!(output, "Found {}\n", summary.format_compact());

        for violation in violations {
            let source_code = violation
                .file
                .as_ref()
                .and_then(|path| std::fs::read_to_string(path.as_ref()).ok())
                .unwrap_or_default();

            let (line, column) = calculate_line_column(&source_code, violation.span.start);
            let (end_line, end_column) = calculate_line_column(&source_code, violation.span.end);

            if let Some(file_path) = &violation.file {
                let _ = writeln!(output, "\x1b[1m{file_path}:{line}:{column}\x1b[0m");
            }

            let diagnostic = ViolationDiagnostic {
                violation: violation.clone(),
                source_code,
                line,
                column,
                end_line,
                end_column,
            };

            let report = Report::new(diagnostic);
            let _ = writeln!(output, "{report:?}");
        }

        let summary = Summary::from_violations(violations);
        let _ = writeln!(output, "\n{}", summary.format_compact());

        output
    }
}

#[derive(Debug, Default)]
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format(&self, violations: &[Violation], _source: &str) -> String {
        let json_violations: Vec<JsonViolation> = violations
            .iter()
            .map(|violation| {
                let source_code = violation
                    .file
                    .as_ref()
                    .and_then(|path| std::fs::read_to_string(path.as_ref()).ok())
                    .unwrap_or_default();

                let (line_start, column_start) =
                    calculate_line_column(&source_code, violation.span.start);
                let (line_end, column_end) =
                    calculate_line_column(&source_code, violation.span.end);

                JsonViolation {
                    rule_id: violation.rule_id.to_string(),
                    severity: violation.severity.to_string(),
                    message: violation.message.to_string(),
                    file: violation
                        .file
                        .as_ref()
                        .map(std::string::ToString::to_string),
                    line_start,
                    line_end,
                    column_start,
                    column_end,
                    offset_start: violation.span.start,
                    offset_end: violation.span.end,
                    suggestion: violation
                        .suggestion
                        .as_ref()
                        .map(std::string::ToString::to_string),
                }
            })
            .collect();

        let summary = Summary::from_violations(violations);
        let output = JsonOutput {
            violations: json_violations,
            summary,
        };

        serde_json::to_string_pretty(&output).unwrap_or_default()
    }
}

/// Calculate line and column number from byte offset in source
/// Returns (line, column) as 1-indexed values
fn calculate_line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (pos, ch) in source.char_indices() {
        if pos >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
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

impl std::error::Error for ViolationDiagnostic {}

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

        Some(Box::new(std::iter::once(LabeledSpan::new(
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
        let mut errors = 0;
        let mut warnings = 0;
        let mut info = 0;

        for violation in violations {
            match violation.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => info += 1,
            }
        }

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
        let mut parts = Vec::new();

        if self.errors > 0 {
            parts.push(format!("{} error(s)", self.errors));
        }
        if self.warnings > 0 {
            parts.push(format!("{} warning(s)", self.warnings));
        }
        if self.info > 0 {
            parts.push(format!("{} info", self.info));
        }

        if parts.is_empty() {
            String::from("0 violations")
        } else {
            parts.join(", ")
        }
    }
}
