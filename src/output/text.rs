use std::{borrow::Cow, collections::HashMap};

use miette::{NamedSource, Report};

use super::{Summary, read_source_code};
use crate::violation::{Fix, Replacement, Violation};

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";
const SEPARATOR_WIDTH: usize = 80;

#[must_use]
pub fn format_text(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return String::from("No violations found!");
    }

    let summary = Summary::from_violations(violations);
    let sources = build_source_cache(violations);

    let violations_output: String = violations
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let file_name = v.file.as_ref().map_or("<stdin>", |f| f.as_str());
            let source = sources.get(file_name).map_or("", String::as_str);
            let is_last = idx == violations.len() - 1;
            format_violation(v, source, file_name, is_last)
        })
        .collect();

    format!("Found {}\n\n{violations_output}", summary.format_compact())
}

fn build_source_cache(violations: &[Violation]) -> HashMap<&str, String> {
    let mut by_file: HashMap<&str, &Violation> = HashMap::new();
    for v in violations {
        let file_name = v.file.as_ref().map_or("<stdin>", |f| f.as_str());
        by_file.entry(file_name).or_insert(v);
    }

    by_file
        .into_iter()
        .map(|(file_name, v)| {
            let source = v
                .source
                .as_ref()
                .map_or_else(|| read_source_code(v.file.as_ref()), ToString::to_string);
            (file_name, source)
        })
        .collect()
}

fn format_violation(
    violation: &Violation,
    source_code: &str,
    file_name: &str,
    is_last: bool,
) -> String {
    let named_source = NamedSource::new(file_name, source_code.to_string());
    let display_violation = with_extended_help(violation, source_code);
    let report = Report::new(display_violation).with_source_code(named_source);

    let separator = if is_last {
        String::new()
    } else {
        format!("\n\n{}\n", "─".repeat(SEPARATOR_WIDTH))
    };

    format!("\n{report:?}{separator}")
}

fn with_extended_help(violation: &Violation, source_code: &str) -> Violation {
    build_help_text(violation, source_code).map_or_else(
        || violation.clone(),
        |text| {
            let mut v = violation.clone();
            v.help = Some(Cow::Owned(text));
            v
        },
    )
}

fn build_help_text(violation: &Violation, source_code: &str) -> Option<String> {
    let parts: Vec<String> = [
        violation.help.as_deref().map(String::from),
        violation
            .fix
            .as_ref()
            .map(|fix| format_fix(fix, source_code, violation.help.is_some()))
            .filter(|s| !s.is_empty()),
        violation
            .doc_url
            .map(|url| format!("See: {}", format_clickable_url(url))),
    ]
    .into_iter()
    .flatten()
    .collect();

    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

fn format_fix(fix: &Fix, source_code: &str, has_help: bool) -> String {
    let diff = fix
        .replacements
        .first()
        .map(|r| format_diff(source_code, r))
        .filter(|d| !d.is_empty());

    match (diff, has_help) {
        (Some(diff_text), true) => diff_text,
        (Some(diff_text), false) => {
            let short = fix
                .explanation
                .split_once(':')
                .map_or(fix.explanation.as_ref(), |(prefix, _)| prefix.trim());
            format!("Available fix: {short}\n{diff_text}")
        }
        (None, _) => format!("Available fix: {}", fix.explanation),
    }
}

fn format_diff(source_code: &str, replacement: &Replacement) -> String {
    let file_span = replacement.file_span();
    let old_text = source_code
        .get(file_span.start..file_span.end)
        .unwrap_or("");
    let new_text = &replacement.replacement_text;

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let format_removed = |line: &str| format!("{RED}  - {line}{RESET}");
    let format_added = |line: &str| format!("{GREEN}  + {line}{RESET}");

    if old_lines.len() > 1 || new_lines.len() > 1 {
        let removed: String = old_lines
            .iter()
            .map(|l| format_removed(l))
            .collect::<Vec<_>>()
            .join("\n");
        let added: String = new_lines
            .iter()
            .map(|l| format_added(l))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{removed}\n{added}")
    } else {
        format!(
            "{}\n{}",
            format_removed(old_text.trim()),
            format_added(new_text.trim())
        )
    }
}

fn format_clickable_url(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}
