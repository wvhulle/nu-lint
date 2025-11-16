use core::{error::Error, iter};
use std::fmt;

use miette::{Diagnostic, LabeledSpan, NamedSource, Report, SourceCode};

use super::{Summary, calculate_line_column, read_source_code};
use crate::violation::Violation;

#[must_use]
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

fn format_violation_text(violation: &Violation, add_separator: bool) -> String {
    let source_code = read_source_code(violation.file.as_ref());

    let named_source = violation.file.as_ref().map_or_else(
        || NamedSource::new("<stdin>", source_code.clone()),
        |file_path| NamedSource::new(file_path.as_ref(), source_code.clone()),
    );

    let diagnostic = ViolationDiagnostic {
        violation: violation.clone(),
        source_code: named_source,
    };

    let report = format!("{:?}", Report::new(diagnostic));

    let separator = if add_separator {
        format!("\n\n{}\n", "─".repeat(80))
    } else {
        String::new()
    };

    format!("\n{report}{separator}")
}

fn format_single_line_diff(
    source_code: &str,
    replacement: &crate::Replacement,
    line_number: usize,
) -> Option<String> {
    let line = source_code.lines().nth(line_number - 1)?;

    let line_start_offset = source_code
        .lines()
        .take(line_number - 1)
        .map(|l| l.len() + 1)
        .sum::<usize>();

    let old_text = source_code
        .get(replacement.span.start..replacement.span.end)
        .unwrap_or("");

    let before = source_code
        .get(line_start_offset..replacement.span.start)
        .unwrap_or("");
    let after = source_code
        .get(replacement.span.end..line_start_offset + line.len())
        .unwrap_or("");

    let old_line = format!("  - {line}");
    let new_line = format!(
        "  + {before}{}{after}",
        replacement.replacement_text
    );

    if old_text.is_empty() && before == line {
        return None;
    }

    Some(format!("{old_line}\n{new_line}"))
}

#[derive(Debug, Clone)]
struct ViolationDiagnostic {
    violation: Violation,
    source_code: NamedSource<String>,
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
            self.violation.lint_level, self.violation.rule_id
        )))
    }

    fn severity(&self) -> Option<miette::Severity> {
        use crate::config::LintLevel;
        Some(match self.violation.lint_level {
            LintLevel::Deny => miette::Severity::Error,
            LintLevel::Warn => miette::Severity::Warning,
            LintLevel::Allow => miette::Severity::Advice,
        })
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        const MAX_LABEL_LENGTH: usize = 150;

        let mut help_text = String::new();

        if let Some(help) = &self.violation.help {
            if help.len() > MAX_LABEL_LENGTH {
                help_text.push_str(help);
            }
        }

        if let Some(fix) = &self.violation.fix {
            if !help_text.is_empty() {
                help_text.push_str("\n\n");
            }
            help_text.push_str("Available fix: ");
            help_text.push_str(&fix.explanation);

            if fix.replacements.len() == 1 {
                let replacement = &fix.replacements[0];
                let source_str = self.source_code.inner();
                let (start_line, _) = calculate_line_column(source_str, replacement.span.start);
                let (end_line, _) = calculate_line_column(source_str, replacement.span.end);

                if start_line == end_line {
                    if let Some(diff) = format_single_line_diff(source_str, replacement, start_line) {
                        help_text.push('\n');
                        help_text.push_str(&diff);
                    }
                }
            }
        }

        (!help_text.is_empty()).then(|| Box::new(help_text) as Box<dyn fmt::Display>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        const MAX_LABEL_LENGTH: usize = 150;

        let span = self.violation.to_source_span();

        let label_text = self.violation.help.as_ref().map_or_else(
            || self.violation.message.to_string(),
            |help| {
                if self.violation.fix.is_none() && help.len() <= MAX_LABEL_LENGTH {
                    help.to_string()
                } else {
                    self.violation.message.to_string()
                }
            },
        );

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
