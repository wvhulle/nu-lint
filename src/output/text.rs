use core::{error::Error, iter};
use std::fmt;

use miette::{Diagnostic, LabeledSpan, Report, SourceCode};

use super::{Summary, calculate_line_column, read_source_code};
use crate::{Fix, violation::Violation};

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

    let (line, column) = calculate_line_column(&source_code, violation.span.start);

    let header = violation.file.as_ref().map_or(String::new(), |file_path| {
        format!("\n\x1b[1;4m{file_path}:{line}:{column}\x1b[0m\n")
    });

    let diagnostic = ViolationDiagnostic {
        violation: violation.clone(),
        source_code: source_code.clone(),
        has_fix: violation.fix.is_some(),
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

fn format_fix_info(fix: &Fix, source_code: &str) -> String {
    let header = format!("\n  \x1b[36mℹ Available fix:\x1b[0m {}", fix.explanation);

    if fix.replacements.is_empty() {
        return header;
    }

    if fix.replacements.len() == 1 {
        let replacement = &fix.replacements[0];
        let (start_line, _start_col) = calculate_line_column(source_code, replacement.span.start);
        let (end_line, _end_col) = calculate_line_column(source_code, replacement.span.end);

        if start_line == end_line
            && let Some(diff) = format_single_line_diff(source_code, replacement, start_line)
        {
            return format!("{header}\n{diff}");
        }
    }

    header
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

    let old_line = format!("  \x1b[31m-\x1b[0m {line}");
    let new_line = format!(
        "  \x1b[32m+\x1b[0m {before}{}{after}",
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
    source_code: String,
    has_fix: bool,
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

        if self.has_fix {
            return None;
        }

        self.violation
            .help
            .as_ref()
            .filter(|s| s.len() > MAX_LABEL_LENGTH)
            .map(|s| Box::new(s.as_ref()) as Box<dyn fmt::Display>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        const MAX_LABEL_LENGTH: usize = 150;

        let span = self.violation.to_source_span();

        let label_text = self.violation.help.as_ref().map_or_else(
            || self.violation.message.to_string(),
            |help| {
                if !self.has_fix && help.len() <= MAX_LABEL_LENGTH {
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
