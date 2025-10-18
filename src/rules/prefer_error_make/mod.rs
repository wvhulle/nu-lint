use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"print\s+(?:-e\s+)?([^\n]+)\s*(?:;|\n)\s*exit\s+(\d+)").unwrap()
    })
}

fn looks_like_error(message: &str, exit_code: i32) -> bool {
    let message_lower = message.to_lowercase();
    let error_indicators = [
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

    // Non-zero exit codes with error-like messages are likely errors
    exit_code != 0
        && error_indicators
            .iter()
            .any(|indicator| message_lower.contains(indicator))
}

fn check(context: &LintContext) -> Vec<Violation> {
    let pat = pattern();

    context.violations_from_regex(pat, "prefer_error_make", Severity::Info, |mat| {
        if let Some(caps) = pat.captures(mat.as_str()) {
            let message = &caps[1].trim_matches('"').trim_matches('\'');
            let exit_code: i32 = caps[2].parse().unwrap_or(1);

            // Only suggest error make for actual error cases
            if looks_like_error(message, exit_code) {
                Some((
                    "Consider using 'error make' instead of 'print' + 'exit' for error conditions"
                        .to_string(),
                    Some(
                        "Use 'error make { msg: \"error message\" }' for better error handling"
                            .to_string(),
                    ),
                ))
            } else {
                None
            }
        } else {
            None
        }
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_error_make",
        RuleCategory::ErrorHandling,
        Severity::Info,
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
