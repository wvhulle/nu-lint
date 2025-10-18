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

fn check(context: &LintContext) -> Vec<Violation> {
    let mut violations = Vec::new();
    let source = context.source;
    let lines: Vec<&str> = source.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        // Check for complex constructs that should be multi-line
        if complex_construct_pattern().is_match(line) && line.len() > 80 {
            let line_start = source[..]
                .lines()
                .take(line_num)
                .map(|l| l.len() + 1)
                .sum::<usize>();
            violations.push(Violation {
                rule_id: "multiline_formatting".into(),
                severity: Severity::Info,
                message: "Complex constructs should use multi-line format for better readability"
                    .to_string()
                    .into(),
                span: nu_protocol::Span::new(line_start, line_start + line.len()),
                suggestion: Some(
                    "Consider breaking this construct across multiple lines"
                        .to_string()
                        .into(),
                ),
                fix: None,
                file: None,
            });
        }

        // Check for lists that should be multi-line
        if multiline_list_pattern().is_match(line) && line.len() > 60 {
            let line_start = source[..]
                .lines()
                .take(line_num)
                .map(|l| l.len() + 1)
                .sum::<usize>();
            violations.push(Violation {
                rule_id: "multiline_formatting".into(),
                severity: Severity::Info,
                message: "Long lists should use multi-line format with each item on a separate \
                          line"
                    .to_string()
                    .into(),
                span: nu_protocol::Span::new(line_start, line_start + line.len()),
                suggestion: Some("Put each list item on a separate line".to_string().into()),
                fix: None,
                file: None,
            });
        }

        // Check for records that should be multi-line
        if multiline_record_pattern().is_match(line) && line.len() > 60 {
            let line_start = source[..]
                .lines()
                .take(line_num)
                .map(|l| l.len() + 1)
                .sum::<usize>();
            violations.push(Violation {
                rule_id: "multiline_formatting".into(),
                severity: Severity::Info,
                message: "Long records should use multi-line format with each field on a separate \
                          line"
                    .to_string()
                    .into(),
                span: nu_protocol::Span::new(line_start, line_start + line.len()),
                suggestion: Some(
                    "Put each record field on a separate line"
                        .to_string()
                        .into(),
                ),
                fix: None,
                file: None,
            });
        }
    }

    violations
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
