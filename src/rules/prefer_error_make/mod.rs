use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"print\s+(?:-e\s+)?([^\n]+)\s*(?:;|\n)\s*exit\s+(\d+)").unwrap()
    })
}

const ERROR_INDICATORS: &[&str] = &[
    "error",
    "failed",
    "cannot",
    "unable",
    "invalid",
    "not found",
    "missing",
    "denied",
    "forbidden",
    "unauthorized",
    "timeout",
    "connection",
    "network",
    "unreachable",
];

fn looks_like_error(message: &str, exit_code: i32) -> bool {
    exit_code != 0
        && ERROR_INDICATORS
            .iter()
            .any(|indicator| message.to_lowercase().contains(indicator))
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let pat = pattern();

    context.violations_from_regex(pat, "prefer_error_make", |mat| {
        pat.captures(mat.as_str()).and_then(|caps| {
            let message = caps[1].trim_matches('"').trim_matches('\'');
            let exit_code: i32 = caps[2].parse().unwrap_or(1);

            looks_like_error(message, exit_code).then(|| {
                (
                    "Consider using 'error make' instead of 'print' + 'exit' for error conditions"
                        .to_string(),
                    Some(
                        "Use 'error make { msg: \"error message\" }' for better error handling"
                            .to_string(),
                    ),
                )
            })
        })
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_error_make",
        RuleCategory::ErrorHandling,
        Severity::Warning,
        "Use 'error make' for custom errors instead of 'print' + 'exit'",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
