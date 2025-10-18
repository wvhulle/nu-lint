use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn ignore_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\|\s*ignore\s*(?:\n|$)").unwrap())
}

fn check(context: &LintContext) -> Vec<Violation> {
    // Pattern: | ignore (but allow external commands with ^)
    let pattern = ignore_pattern();

    pattern
        .find_iter(context.source)
        .filter_map(|mat| {
            // Get some context before the match to see if it's an external command
            let context_start = mat.start().saturating_sub(50);
            let context_str = &context.source[context_start..mat.start()];

            // If it's an external command (^), it's more acceptable
            let is_external = context_str.contains('^');

            if is_external {
                None
            } else {
                Some(Violation {
                    rule_id: "discourage_bare_ignore".into(),
                    severity: Severity::Info,
                    message: "Piping to 'ignore' suppresses output without error handling"
                        .to_string()
                        .into(),
                    span: nu_protocol::Span::new(mat.start(), mat.end()),
                    suggestion: Some(
                        "Consider: 'do -i { ... }' for error suppression or handle errors \
                         explicitly"
                            .to_string()
                            .into(),
                    ),
                    fix: None,
                    file: None,
                })
            }
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "discourage_bare_ignore",
        RuleCategory::ErrorHandling,
        Severity::Info,
        "Using '| ignore' may hide errors - consider explicit error handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
