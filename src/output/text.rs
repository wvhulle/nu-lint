use std::{borrow::Cow, collections::HashMap};

use miette::{NamedSource, Report};

use super::{Summary, read_source_code};
use crate::violation::{Fix, Replacement, Violation};

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

    // Build extended help text including fix diffs
    let help = build_help_text(violation, source_code);

    // Clone violation and add the extended help
    let display_violation = if help.is_some() {
        let mut v = violation.clone();
        v.help = help.map(Cow::Owned);
        v
    } else {
        violation.clone()
    };

    let report = Report::new(display_violation).with_source_code(named_source);

    let separator = if add_separator {
        format!("\n\n{}\n", "─".repeat(80))
    } else {
        String::new()
    };

    format!("\n{report:?}{separator}")
}

fn build_help_text(violation: &Violation, source_code: &str) -> Option<String> {
    let mut help_text = String::new();

    if let Some(help) = &violation.help {
        help_text.push_str(help);
    }

    if let Some(fix) = &violation.fix {
        let fix_text = format_fix_help(fix, source_code, violation.help.is_some());
        if !fix_text.is_empty() {
            if !help_text.is_empty() {
                help_text.push_str("\n\n");
            }
            help_text.push_str(&fix_text);
        }
    }

    if let Some(doc_url) = &violation.doc_url {
        if !help_text.is_empty() {
            help_text.push_str("\n\n");
        }
        help_text.push_str("See: ");
        help_text.push_str(&format_clickable_url(doc_url));
    }

    (!help_text.is_empty()).then_some(help_text)
}

// ANSI color codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

fn format_replacement_diff(source_code: &str, replacement: &Replacement) -> String {
    let old_text = source_code
        .get(replacement.span.start..replacement.span.end)
        .unwrap_or("");

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = replacement.replacement_text.lines().collect();

    if old_lines.len() > 1 || new_lines.len() > 1 {
        let before = old_lines
            .iter()
            .map(|line| format!("{RED}  - {line}{RESET}"))
            .collect::<Vec<_>>()
            .join("\n");
        let after = new_lines
            .iter()
            .map(|line| format!("{GREEN}  + {line}{RESET}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{before}\n{after}")
    } else {
        format!(
            "{RED}  - {}{RESET}\n{GREEN}  + {}{RESET}",
            old_text.trim(),
            replacement.replacement_text.trim()
        )
    }
}

fn format_fix_help(fix: &Fix, source_code: &str, has_help_text: bool) -> String {
    let diff = fix
        .replacements
        .first()
        .map(|r| format_replacement_diff(source_code, r))
        .filter(|d| !d.is_empty());

    match diff {
        Some(diff_text) if has_help_text => {
            // Help text already explains the fix, just show the diff
            diff_text
        }
        Some(diff_text) => {
            // No help text, show short description with diff
            let short_explanation = fix
                .explanation
                .split_once(':')
                .map_or(fix.explanation.as_ref(), |(prefix, _)| prefix.trim());
            format!("Available fix: {short_explanation}\n{diff_text}")
        }
        None => {
            // No diff available, show the full explanation
            format!("Available fix: {}", fix.explanation)
        }
    }
}

fn format_clickable_url(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}
