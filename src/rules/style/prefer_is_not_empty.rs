use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;
use std::sync::OnceLock;

pub struct PreferIsNotEmpty;

impl PreferIsNotEmpty {
    pub fn new() -> Self {
        Self
    }

    fn not_is_empty_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"not\s+\([^)]*\|\s*is-empty\s*\)").unwrap())
    }

    fn if_not_is_empty_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"if\s+not\s+\(\$[^)]*\|\s*is-empty\s*\)").unwrap())
    }
}

impl Default for PreferIsNotEmpty {
    fn default() -> Self {
        Self::new()
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
        let mut violations = Vec::new();

        // Pattern: not (... | is-empty)
        violations.extend(context.violations_from_regex(
            Self::not_is_empty_pattern(),
            self.id(),
            self.severity(),
            "Double negative 'not ... is-empty' reduces readability",
            Some("Use 'is-not-empty' for clearer intent".to_string()),
        ));

        // Also match: if not ($var | is-empty)
        violations.extend(context.violations_from_regex(
            Self::if_not_is_empty_pattern(),
            self.id(),
            self.severity(),
            "Use 'is-not-empty' instead of 'not ... is-empty'",
            Some("Replace 'if not ($x | is-empty)' with 'if ($x | is-not-empty)'".to_string()),
        ));

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_is_empty_detected() {
        let rule = PreferIsNotEmpty::new();

        let bad_code = "if not ($list | is-empty) { echo 'has items' }";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect 'not ... is-empty'"
        );
    }

    #[test]
    fn test_is_not_empty_not_flagged() {
        let rule = PreferIsNotEmpty::new();

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
        let rule = PreferIsNotEmpty::new();

        let good_code = "if ($list | is-empty) { echo 'no items' }";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag plain 'is-empty'"
        );
    }
}
