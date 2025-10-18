use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

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

            violations.push(Violation {
                rule_id: "no_trailing_spaces".into(),
                severity: Severity::Warning,
                message: format!("Line {} has trailing whitespace", line_num + 1).into(),
                span: nu_protocol::Span::new(violation_start, violation_end),
                suggestion: Some("Remove trailing spaces".into()),
                fix: None,
                file: None,
            });
        }

        // Update byte offset for next line (including newline character)
        byte_offset += line.len() + 1;
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "no_trailing_spaces",
        RuleCategory::Formatting,
        Severity::Warning,
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
