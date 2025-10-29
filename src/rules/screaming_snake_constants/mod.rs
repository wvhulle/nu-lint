use std::sync::OnceLock;

use heck::ToShoutySnakeCase;
use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn screaming_snake_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap())
}

fn const_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\bconst\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap())
}

fn is_valid_screaming_snake(name: &str) -> bool {
    screaming_snake_pattern().is_match(name)
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    const_pattern()
        .captures_iter(context.source)
        .filter_map(|cap| {
            let const_match = cap.get(1)?;
            let const_name = const_match.as_str();

            if is_valid_screaming_snake(const_name) {
                None
            } else {
                Some(
                    RuleViolation::new_dynamic(
                        "screaming_snake_constants",
                        format!(
                            "Constant '{const_name}' should use SCREAMING_SNAKE_CASE naming \
                             convention"
                        ),
                        nu_protocol::Span::new(const_match.start(), const_match.end()),
                    )
                    .with_suggestion_dynamic(format!(
                        "Consider renaming to: {}",
                        const_name.to_shouty_snake_case()
                    )),
                )
            }
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "screaming_snake_constants",
        RuleCategory::Naming,
        Severity::Info,
        "Constants should use SCREAMING_SNAKE_CASE naming convention",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
