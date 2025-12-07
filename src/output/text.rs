use std::{borrow::Cow, collections::HashMap, error::Error, fmt, iter};

use miette::{Diagnostic, LabeledSpan, NamedSource, Report, Severity, SourceCode};

use super::{Summary, read_source_code};
use crate::{
    ast::span::SpanExt,
    violation::{Fix, Replacement, Violation},
};

#[must_use]
pub fn format_text(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return String::from("No violations found!");
    }

    let summary = Summary::from_violations(violations);
    let header = format!("Found {}\n\n", summary.format_compact());

    // Group violations by file for efficient source reading
    let mut by_file: HashMap<&str, Vec<&Violation>> = HashMap::new();
    for v in violations {
        let file_name = v.file.as_deref().unwrap_or("<stdin>");
        by_file.entry(file_name).or_default().push(v);
    }

    // Pre-read source code for each file
    let sources: HashMap<&str, String> = by_file
        .keys()
        .map(|&file_name| {
            let source = by_file
                .get(file_name)
                .and_then(|vs| vs.first())
                .and_then(|v| v.source.as_ref())
                .map_or_else(
                    || read_source_code(Some(&Cow::Borrowed(file_name))),
                    ToString::to_string,
                );
            (file_name, source)
        })
        .collect();

    // Format each violation
    let violations_output: String = violations
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let file_name = v.file.as_deref().unwrap_or("<stdin>");
            let source = sources.get(file_name).cloned().unwrap_or_default();
            format_violation_text(v, &source, file_name, idx < violations.len() - 1)
        })
        .collect();

    format!("{header}{violations_output}")
}

fn format_violation_text(
    violation: &Violation,
    source_code: &str,
    file_name: &str,
    add_separator: bool,
) -> String {
    let named_source = NamedSource::new(file_name, source_code.to_string());

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

fn format_replacement_diff(source_code: &str, replacement: &Replacement) -> String {
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

/// Format a URL as a clickable terminal hyperlink (OSC 8)
fn format_clickable_url(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
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
            self.violation.lint_level,
            self.violation.rule_id.as_deref().unwrap_or("unknown")
        )))
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.violation.lint_level.into())
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
        // Use unlabeled span for primary to avoid duplicating the message
        // (message is already shown via Display::fmt above the source)
        let primary = self.violation.span.unlabeled();
        let secondary = self.violation.labels.iter().cloned();

        Some(Box::new(iter::once(primary).chain(secondary)))
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source_code as &dyn SourceCode)
    }
}
