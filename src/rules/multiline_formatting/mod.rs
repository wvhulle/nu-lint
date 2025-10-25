use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn complex_construct_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?:def|export def|if|match|for|while|try)\s+[^{]*\{[^}]*\}.*$")
            .unwrap()
    })
}

fn multiline_list_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\[[^\[\]]*[,\s][^\[\]]*\]").unwrap())
}

fn multiline_record_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\{[^{}]*:[^{}]*[,\s][^{}]*\}").unwrap())
}

struct LineCheck {
    pattern: fn() -> &'static Regex,
    max_length: usize,
    message: &'static str,
    suggestion: &'static str,
}

const LINE_CHECKS: &[LineCheck] = &[
    LineCheck {
        pattern: complex_construct_pattern,
        max_length: 80,
        message: "Complex constructs should use multi-line format for better readability",
        suggestion: "Consider breaking this construct across multiple lines",
    },
    LineCheck {
        pattern: multiline_list_pattern,
        max_length: 60,
        message: "Long lists should use multi-line format with each item on a separate line",
        suggestion: "Put each list item on a separate line",
    },
    LineCheck {
        pattern: multiline_record_pattern,
        max_length: 60,
        message: "Long records should use multi-line format with each field on a separate line",
        suggestion: "Put each record field on a separate line",
    },
];

fn check(context: &LintContext) -> Vec<Violation> {
    let source = context.source;
    let lines: Vec<&str> = source.lines().collect();

    lines
        .iter()
        .enumerate()
        .flat_map(|(line_num, line)| {
            LINE_CHECKS.iter().filter_map(move |check| {
                if (check.pattern)().is_match(line) && line.len() > check.max_length {
                    let line_start = source[..]
                        .lines()
                        .take(line_num)
                        .map(|l| l.len() + 1)
                        .sum::<usize>();
                    Some(Violation {
                        rule_id: "multiline_formatting".into(),
                        severity: Severity::Info,
                        message: check.message.into(),
                        span: nu_protocol::Span::new(line_start, line_start + line.len()),
                        suggestion: Some(check.suggestion.into()),
                        fix: None,
                        file: None,
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "multiline_formatting",
        RuleCategory::Formatting,
        Severity::Info,
        "Prefer multi-line format for complex constructs and longer lists/records",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
