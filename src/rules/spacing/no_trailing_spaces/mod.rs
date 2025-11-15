use std::sync::OnceLock;

use regex::Regex;

use crate::{context::LintContext, rule::Rule, violation::Violation};
fn trailing_space_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"[ \t]+$").unwrap())
}
fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    let source = context.source;
    let lines: Vec<&str> = source.lines().collect();
    let mut byte_offset = 0;
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(m) = trailing_space_pattern().find(line) {
            let violation_start = byte_offset + m.start();
            let violation_end = byte_offset + m.end();
            violations.push(
                Violation::new_dynamic(
                    "no_trailing_spaces",
                    format!("Line {} has trailing whitespace", line_num + 1),
                    nu_protocol::Span::new(violation_start, violation_end),
                )
                .with_suggestion_static("Remove trailing spaces"),
            );
        }
        // Update byte offset for next line (including newline character)
        byte_offset += line.len() + 1;
    }
    violations
}
pub const fn rule() -> Rule {
    Rule::new(
        "no_trailing_spaces",
        "Eliminate trailing spaces at the end of lines",
        check,
    )
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
