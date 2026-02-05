use std::{collections::HashMap, iter::once};

use miette::Severity;

use super::read_source_code;
use crate::violation::Violation;

/// Format violations in compact one-line-per-violation style (gcc/eslint).
///
/// Output format: `file:line:col: severity(rule_id): message`
#[must_use]
pub fn format_compact(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return String::from("No violations found!");
    }

    let sources = build_source_cache(violations);

    violations
        .iter()
        .map(|v| {
            let file_name = v.file.as_ref().map_or("<stdin>", |f| f.as_str());
            let source = sources.get(file_name).map_or("", String::as_str);
            let span = v.file_span();
            let (line, col) = byte_offset_to_line_col(source, span.start);
            let severity = severity_label(v.lint_level);
            let rule_id = v.rule_id.as_deref().unwrap_or("unknown");
            format!(
                "{file_name}:{line}:{col}: {severity}({rule_id}): {}",
                v.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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

/// Convert a byte offset in `source` to a 1-based `(line, col)` pair.
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());

    let line_starts: Vec<usize> = once(0)
        .chain(
            source
                .bytes()
                .enumerate()
                .filter_map(|(i, b)| (b == b'\n').then_some(i + 1)),
        )
        .collect();

    let line_index = line_starts
        .partition_point(|&start| start <= offset)
        .saturating_sub(1);
    let col = offset - line_starts[line_index];

    (line_index + 1, col + 1)
}

const fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Advice => "hint",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_offset_at_start() {
        assert_eq!(byte_offset_to_line_col("hello\nworld", 0), (1, 1));
    }

    #[test]
    fn byte_offset_mid_first_line() {
        assert_eq!(byte_offset_to_line_col("hello\nworld", 3), (1, 4));
    }

    #[test]
    fn byte_offset_at_newline() {
        // offset 5 is the '\n' character itself — still on line 1
        assert_eq!(byte_offset_to_line_col("hello\nworld", 5), (1, 6));
    }

    #[test]
    fn byte_offset_start_of_second_line() {
        assert_eq!(byte_offset_to_line_col("hello\nworld", 6), (2, 1));
    }

    #[test]
    fn byte_offset_past_end_clamped() {
        assert_eq!(byte_offset_to_line_col("hello", 999), (1, 6));
    }

    #[test]
    fn byte_offset_empty_source() {
        assert_eq!(byte_offset_to_line_col("", 0), (1, 1));
    }
}
