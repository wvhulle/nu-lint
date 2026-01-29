use std::{cmp::Reverse, collections::HashMap, fmt::Write};

use miette::{NamedSource, Report};

use super::{Summary, read_source_code};
use crate::{
    format::{format_clickable_url, format_diff_inline},
    violation::{ExternalDetection, Fix, Replacement, Violation},
};

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
