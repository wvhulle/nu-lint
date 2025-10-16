use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
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
    fn id(&self) -> &str {
        "S010"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
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
                Some(suggestion.to_string()),
            )
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_is_empty_detected() {
        let rule = PreferIsNotEmpty::default();

        let bad_code = "if not ($list | is-empty) { echo 'has items' }";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect 'not ... is-empty'"
        );
    }

    #[test]
    fn test_is_not_empty_not_flagged() {
        let rule = PreferIsNotEmpty::default();

        let good_code = "if ($list | is-not-empty) { echo 'has items' }";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag 'is-not-empty'"
        );
    }

    #[test]
    fn test_plain_is_empty_not_flagged() {
        let rule = PreferIsNotEmpty::default();

        let good_code = "if ($list | is-empty) { echo 'no items' }";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag plain 'is-empty'"
        );
    }
}
