use core::{error::Error, iter};
use std::fmt;

use miette::{Diagnostic, LabeledSpan, NamedSource, Report, SourceCode};

use super::{Summary, read_source_code};
use crate::violation::{Fix, Violation};

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
    // Use source from violation if available (stdin), otherwise read from file
    let source_code = violation.source.as_ref().map_or_else(
        || read_source_code(violation.file.as_ref()),
        ToString::to_string,
    );

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

fn format_replacement_diff(source_code: &str, replacement: &crate::Replacement) -> String {
    let old_text = source_code
        .get(replacement.span.start..replacement.span.end)
        .unwrap_or("");

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = replacement.replacement_text.lines().collect();

    if old_lines.len() > 1 || new_lines.len() > 1 {
        let before = old_lines
            .iter()
            .map(|line| format!("  - {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        let after = new_lines
            .iter()
            .map(|line| format!("  + {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{before}\n{after}")
    } else {
        format!(
            "  - {}\n  + {}",
            old_text.trim(),
            replacement.replacement_text.trim()
        )
    }
}

fn format_fix_help(fix: &Fix, source_code: &str) -> String {
    let mut help_text = String::from("Available fix: ");
    help_text.push_str(&fix.explanation);

    if let Some(replacement) = fix.replacements.first() {
        let diff = format_replacement_diff(source_code, replacement);
        if !diff.is_empty() {
            help_text.push('\n');
            help_text.push_str(&diff);
        }
    }

    help_text
}

#[derive(Debug, Clone)]
struct ViolationDiagnostic {
    violation: Violation,
    source_code: NamedSource<String>,
}

/// Format a URL as a clickable terminal hyperlink (OSC 8)
fn format_clickable_url(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
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
        let mut help_text = String::new();

        if let Some(help) = &self.violation.help {
            help_text.push_str(help);
        }

        if let Some(fix) = &self.violation.fix {
            if !help_text.is_empty() {
                help_text.push_str("\n\n");
            }
            help_text.push_str(&format_fix_help(fix, self.source_code.inner()));
        }

        if let Some(doc_url) = &self.violation.doc_url {
            if !help_text.is_empty() {
                help_text.push_str("\n\n");
            }
            help_text.push_str("See: ");
            help_text.push_str(&format_clickable_url(doc_url));
        }

        (!help_text.is_empty()).then(|| Box::new(help_text) as Box<dyn fmt::Display>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let span = self.violation.to_source_span();
        let label_text = self.violation.message.to_string();

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
