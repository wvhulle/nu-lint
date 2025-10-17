use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

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

impl RuleMetadata for PreferIsNotEmpty {
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
}

impl RegexRule for PreferIsNotEmpty {
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
            context.violations_from_regex_if(pattern, self.id(), self.severity(), |_| {
                Some((msg.to_string(), Some(suggestion.to_string())))
            })
        })
        .collect()
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
