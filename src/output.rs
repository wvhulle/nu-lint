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

        for violation in violations {
            let source_code = violation
                .file
                .as_ref()
                .and_then(|path| std::fs::read_to_string(path.as_ref()).ok())
                .unwrap_or_default();

            let (line, column) = calculate_line_column(&source_code, violation.span.start);

            if let Some(file_path) = &violation.file {
                writeln!(output, "\x1b[1m{file_path}:{line}:{column}\x1b[0m").unwrap();
            }

            let diagnostic = ViolationDiagnostic {
                violation: violation.clone(),
                source_code,
            };

            let report = Report::new(diagnostic);
            writeln!(output, "{report:?}").unwrap();
        }

        let summary = Summary::from_violations(violations);
        writeln!(
            output,
            "\n{} error(s), {} warning(s), {} info",
            summary.errors, summary.warnings, summary.info
        )
        .unwrap();

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

                let (line, column) = calculate_line_column(&source_code, violation.span.start);

                JsonViolation {
                    rule_id: violation.rule_id.to_string(),
                    severity: violation.severity.to_string(),
                    message: violation.message.to_string(),
                    file: violation.file.as_ref().map(std::string::ToString::to_string),
                    line,
                    column,
                    suggestion: violation.suggestion.as_ref().map(std::string::ToString::to_string),
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
        Some(Box::new(std::iter::once(LabeledSpan::new(
            Some(self.violation.message.to_string()),
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
    pub line: usize,
    pub column: usize,
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
        let (errors, warnings, info) =
            violations
                .iter()
                .fold((0, 0, 0), |(e, w, i), v| match v.severity {
                    Severity::Error => (e + 1, w, i),
                    Severity::Warning => (e, w + 1, i),
                    Severity::Info => (e, w, i + 1),
                });

        Self {
            errors,
            warnings,
            info,
            files_checked: 1,
        }
    }
}
