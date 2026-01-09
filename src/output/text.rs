use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Write},
    iter,
};

use miette::{Diagnostic, LabeledSpan, NamedSource, Report, Severity};

use super::{Summary, read_source_code};
use crate::violation::{ExternalDetection, Fix, Replacement, Violation};

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

    let mut output = String::new();
    output.push_str("\n  Related locations in external files:\n");

    for detection in detections {
        let source = NamedSource::new(&detection.file, detection.source.clone());
        let label = LabeledSpan::at(
            detection.span.start..detection.span.end,
            detection.label.as_deref().unwrap_or("here"),
        );

        let diagnostic = ExternalDiagnostic {
            message: detection.message.clone(),
            label,
        };

        let report = Report::new(diagnostic).with_source_code(source);
        let _ = write!(output, "{report:?}");
    }

    output
}

/// Helper diagnostic for rendering external file locations
#[derive(Debug)]
struct ExternalDiagnostic {
    message: String,
    label: LabeledSpan,
}

impl fmt::Display for ExternalDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ExternalDiagnostic {}

impl Diagnostic for ExternalDiagnostic {
    fn severity(&self) -> Option<Severity> {
        Some(Severity::Advice)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(iter::once(self.label.clone())))
    }
}

fn with_extended_help(violation: &Violation, source_code: &str) -> Violation {
    build_help_text(violation, source_code).map_or_else(
        || violation.clone(),
        |text| {
            let mut v = violation.clone();
            v.long_description = Some(text);
            v
        },
    )
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
    let diff = (!diff.is_empty()).then_some(diff);

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

fn format_combined_diff(source_code: &str, replacements: &[Replacement]) -> String {
    if replacements.is_empty() {
        return String::new();
    }

    // Sort replacements by start position (descending) to apply from end to start
    let mut sorted_replacements: Vec<_> = replacements.iter().collect();
    sorted_replacements.sort_by(|a, b| b.file_span().start.cmp(&a.file_span().start));

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
    let mut new_source = source_code.to_string();
    for replacement in &sorted_replacements {
        let file_span = replacement.file_span();
        new_source.replace_range(
            file_span.start..file_span.end,
            &replacement.replacement_text,
        );
    }

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

    format_diff_text(old_text, new_text)
}

fn format_diff_text(old_text: &str, new_text: &str) -> String {
    if old_text == new_text {
        return String::new();
    }

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
