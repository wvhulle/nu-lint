use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Default)]
pub struct PreferIsNotEmpty;

impl PreferIsNotEmpty {
    fn not_is_empty_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"not\s+\([^)]*\|\s*is-empty\s*\)").unwrap())
    }

    fn if_not_is_empty_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"if\s+not\s+\(\$[^)]*\|\s*is-empty\s*\)").unwrap())
    }
}

impl Rule for PreferIsNotEmpty {
    fn id(&self) -> &'static str {
        "prefer_is_not_empty"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Use 'is-not-empty' instead of 'not ... is-empty' for better readability"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        [
            (
                Self::not_is_empty_pattern(),
                "Double negative 'not ... is-empty' reduces readability",
                "Use 'is-not-empty' for clearer intent",
            ),
            (
                Self::if_not_is_empty_pattern(),
                "Use 'is-not-empty' instead of 'not ... is-empty'",
                "Replace 'if not ($x | is-empty)' with 'if ($x | is-not-empty)'",
            ),
        ]
        .into_iter()
        .flat_map(|(pattern, msg, suggestion)| {
            context.violations_from_regex(
                pattern,
                self.id(),
                self.severity(),
                msg,
                Some(suggestion),
            )
        })
        .collect()
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
#[cfg(test)]
mod generated_fix;