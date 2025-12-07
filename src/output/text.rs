use std::io::Cursor;

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

use super::{Summary, read_source_code};
use crate::{
    config::LintLevel,
    violation::{Fix, Violation},
};

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

    format!("{header}{violations_output}")
}

const fn lint_level_to_report_kind(level: LintLevel) -> ReportKind<'static> {
    match level {
        LintLevel::Deny => ReportKind::Error,
        LintLevel::Warn => ReportKind::Warning,
        LintLevel::Allow => ReportKind::Advice,
    }
}

fn format_violation_text(violation: &Violation, add_separator: bool) -> String {
    let source_code = violation.source.as_ref().map_or_else(
        || read_source_code(violation.file.as_ref()),
        ToString::to_string,
    );

    let file_name: &str = violation.file.as_ref().map_or("<stdin>", |f| f.as_ref());

    let mut colors = ColorGenerator::new();
    let primary_color = colors.next();

    let report_kind = lint_level_to_report_kind(violation.lint_level);
    let code = violation.rule_id.as_deref().unwrap_or("unknown");

    let mut report_builder = Report::build(
        report_kind,
        (file_name, violation.span.start..violation.span.end),
    )
    .with_code(code)
    .with_message(&violation.message);

    report_builder = report_builder.with_label(
        Label::new((file_name, violation.span.start..violation.span.end))
            .with_message(&violation.message)
            .with_color(primary_color),
    );

    for label in &violation.labels {
        let color = colors.next();
        let ariadne_label =
            Label::new((file_name, label.span.start..label.span.end)).with_color(color);

        report_builder = report_builder.with_label(if let Some(text) = &label.text {
            ariadne_label.with_message(text.as_ref())
        } else {
            ariadne_label
        });
    }

    if let Some(help) = &violation.help {
        report_builder = report_builder.with_help(help.as_ref());
    }

    if let Some(fix) = &violation.fix {
        let fix_note = format_fix_note(fix, &source_code);
        report_builder = report_builder.with_note(fix_note);
    }

    if let Some(doc_url) = &violation.doc_url {
        let url_note = format!("See: {}", format_clickable_url(doc_url));
        report_builder = report_builder.with_note(url_note);
    }

    let report = report_builder.finish();

    let mut output = Cursor::new(Vec::new());
    report
        .write((file_name, Source::from(&source_code)), &mut output)
        .unwrap_or_default();

    let report_str = String::from_utf8(output.into_inner()).unwrap_or_default();

    let separator = if add_separator {
        format!("\n{}\n", "─".repeat(80))
    } else {
        String::new()
    };

    format!("\n{report_str}{separator}")
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

fn format_fix_note(fix: &Fix, source_code: &str) -> String {
    let mut note = format!("Available fix: {}", fix.explanation);

    if let Some(replacement) = fix.replacements.first() {
        let diff = format_replacement_diff(source_code, replacement);
        if !diff.is_empty() {
            note.push('\n');
            note.push_str(&diff);
        }
    }

    note
}

/// Format a URL as a clickable terminal hyperlink (OSC 8)
fn format_clickable_url(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}
