use std::{cmp::Reverse, collections::HashMap, fmt::Write};

use miette::{NamedSource, Report};
use owo_colors::OwoColorize;

use super::{Summary, read_source_code};
use crate::violation::{ExternalDetection, Fix, Replacement, Violation};

const SEPARATOR_WIDTH: usize = 80;

#[must_use]
pub fn format_pretty(violations: &[Violation]) -> String {
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
    violations.iter().fold(HashMap::new(), |mut cache, v| {
        let file_name = v.file.as_ref().map_or("<stdin>", |f| f.as_str());
        cache.entry(file_name).or_insert_with(|| {
            v.source
                .as_ref()
                .map_or_else(|| read_source_code(v.file.as_ref()), ToString::to_string)
        });
        cache
    })
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

    // Format external detections if present
    let external_output = format_external_detections(&violation.external_detections);

    let separator = if is_last {
        String::new()
    } else {
        format!("\n\n{}\n", "─".repeat(SEPARATOR_WIDTH))
    };

    format!("\n{report:?}{external_output}{separator}")
}

/// Format external detections as additional diagnostic output
fn format_external_detections(detections: &[ExternalDetection]) -> String {
    if detections.is_empty() {
        return String::new();
    }

    detections.iter().fold(
        String::from("\n  Related locations in external files:\n"),
        |mut output, detection| {
            let source = NamedSource::new(&detection.file, detection.source.clone());
            let report = Report::new(detection.clone()).with_source_code(source);
            let _ = write!(output, "{report:?}");
            output
        },
    )
}

fn with_extended_help(violation: &Violation, source_code: &str) -> Violation {
    let mut v = violation.clone();
    if let Some(text) = build_help_text(violation, source_code) {
        v.long_description = Some(text);
    }
    v
}

fn build_help_text(violation: &Violation, source_code: &str) -> Option<String> {
    let parts: Vec<String> = [
        violation.long_description.as_deref().map(String::from),
        violation
            .fix
            .as_ref()
            .map(|fix| format_fix(fix, source_code, violation.long_description.is_some()))
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
    let diff = format_combined_diff(source_code, &fix.replacements);

    if diff.is_empty() {
        return format!("Available fix: {}", fix.explanation);
    }

    if has_help {
        return diff;
    }

    let short = fix
        .explanation
        .split_once(':')
        .map_or(fix.explanation.as_ref(), |(prefix, _)| prefix.trim());
    format!("Available fix: {short}\n{diff}")
}

fn format_combined_diff(source_code: &str, replacements: &[Replacement]) -> String {
    if replacements.is_empty() {
        return String::new();
    }

    // Sort replacements by start position (descending) to apply from end to start
    let mut sorted_replacements: Vec<_> = replacements.iter().collect();
    sorted_replacements.sort_by_key(|b| Reverse(b.file_span().start));

    // Find the span that encompasses all replacements
    let min_start = replacements
        .iter()
        .map(|r| r.file_span().start)
        .min()
        .unwrap_or(0);
    let max_end = replacements
        .iter()
        .map(|r| r.file_span().end)
        .max()
        .unwrap_or(0);

    // Get the original text for the affected region
    let old_text = source_code.get(min_start..max_end).unwrap_or("");

    // Apply all replacements to get the new text
    let new_source = sorted_replacements
        .iter()
        .fold(source_code.to_string(), |mut s, r| {
            let span = r.file_span();
            s.replace_range(span.start..span.end, &r.replacement_text);
            s
        });

    // Calculate the new end position after replacements
    #[allow(
        clippy::cast_possible_wrap,
        reason = "replacement text lengths are bounded"
    )]
    let length_delta: isize = replacements
        .iter()
        .map(|r| {
            let file_span = r.file_span();
            r.replacement_text.len() as isize - (file_span.end - file_span.start) as isize
        })
        .sum();

    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        reason = "result is clamped to valid range"
    )]
    let new_end = (max_end as isize + length_delta).max(min_start as isize) as usize;

    let new_text = new_source.get(min_start..new_end).unwrap_or("");

    format_diff_inline(old_text, new_text)
}

// --- Diff formatting utilities (previously in src/format.rs) ---

/// Format a diff between old and new text for inline display (violation
/// previews). Shows all removed lines in red, then all added lines in green.
fn format_diff_inline(old_text: &str, new_text: &str) -> String {
    if old_text == new_text {
        return String::new();
    }

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let format_removed = |line: &str| format!("  - {}", line.red());
    let format_added = |line: &str| format!("  + {}", line.green());

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

/// Format a diff between old and new text with line numbers (fix application
/// output). Shows only changed lines with their line numbers.
#[must_use]
pub fn format_diff_context(original: &str, fixed: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let fixed_lines: Vec<&str> = fixed.lines().collect();

    if original_lines == fixed_lines {
        return String::from("No changes\n");
    }

    let mut output = String::new();
    let max_lines = original_lines.len().max(fixed_lines.len());

    for i in 0..max_lines {
        let orig = original_lines.get(i);
        let fix = fixed_lines.get(i);

        match (orig, fix) {
            (Some(o), Some(f)) if o != f => {
                writeln!(output, "{}", format!("-{:>4} {o}", i + 1).red()).unwrap();
                writeln!(output, "{}", format!("+{:>4} {f}", i + 1).green()).unwrap();
            }
            (Some(o), None) => {
                writeln!(output, "{}", format!("-{:>4} {o}", i + 1).red()).unwrap();
            }
            (None, Some(f)) => {
                writeln!(output, "{}", format!("+{:>4} {f}", i + 1).green()).unwrap();
            }
            _ => {}
        }
    }

    if output.is_empty() {
        String::from("No changes\n")
    } else {
        output
    }
}

/// Format a URL as a clickable terminal hyperlink (OSC 8 escape sequence).
fn format_clickable_url(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}
