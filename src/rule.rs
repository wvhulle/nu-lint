use core::hash::Hasher;
use std::hash::Hash;

use crate::{context::LintContext, violation::Violation};

/// Lint sets (collections of rules, similar to Clippy's lint groups)
#[derive(Debug, Clone, Copy)]

/// A concrete rule struct that wraps the check function
pub struct Rule {
    pub id: &'static str,
    pub explanation: &'static str,
    pub(crate) check: fn(&LintContext) -> Vec<Violation>,
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Rule {}

impl Rule {
    /// Create a new rule
    pub(crate) const fn new(
        id: &'static str,
        explanation: &'static str,
        check: fn(&LintContext) -> Vec<Violation>,
    ) -> Self {
        Self {
            id,
            explanation,
            check,
        }
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule finds violations in the given code
    pub fn assert_detects(&self, code: &str) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations in code, but found none",
            self.id
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule finds no violations in the given code
    pub fn assert_ignores(&self, code: &str) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert!(
            violations.is_empty(),
            "Expected rule '{}' to ignore code, but found {} violations",
            self.id,
            violations.len()
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule finds at least the expected number of
    /// violations
    pub fn assert_violation_count(&self, code: &str, expected_min: usize) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert!(
            violations.len() >= expected_min,
            "Expected rule '{}' to find at least {} violations, but found {}",
            self.id,
            expected_min,
            violations.len()
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule finds exactly the expected number of
    /// violations
    pub fn assert_violation_count_exact(&self, code: &str, expected: usize) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert_eq!(
            violations.len(),
            expected,
            "Expected rule '{}' to find exactly {} violation(s), but found {}",
            self.id,
            expected,
            violations.len()
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule generates a fix with replacement text
    /// containing the expected string
    pub fn assert_fix_contains(&self, code: &str, expected_text: &str) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations, but found none",
            self.id
        );

        let fix = violations[0]
            .fix
            .as_ref()
            .expect("Expected violation to have a fix");

        assert!(
            !fix.replacements.is_empty(),
            "Expected fix to have replacements"
        );

        let replacement_text = &fix.replacements[0].replacement_text;
        assert!(
            replacement_text.contains(expected_text),
            "Expected fix replacement text to contain '{expected_text}', but got: \
             {replacement_text}"
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that applying the fix produces the expected code
    pub fn assert_fix(&self, bad_code: &str, expected_code: &str) {
        let violations =
            LintContext::test_with_parsed_source(bad_code, |context| (self.check)(&context));
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations, but found none",
            self.id
        );

        let fix = violations[0]
            .fix
            .as_ref()
            .expect("Expected violation to have a fix");

        assert!(
            !fix.replacements.is_empty(),
            "Expected fix to have replacements"
        );

        let replacement_text = &fix.replacements[0].replacement_text;
        assert_eq!(
            replacement_text.as_ref(),
            expected_code,
            "Expected fix to produce exact code"
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule generates a fix with explanation
    /// containing the expected string
    pub fn assert_fix_explanation_contains(&self, code: &str, expected_text: &str) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations, but found none",
            self.id
        );

        let fix = violations[0]
            .fix
            .as_ref()
            .expect("Expected violation to have a fix");

        let explanation = &fix.explanation;
        assert!(
            explanation.contains(expected_text),
            "Expected fix explanation to contain '{expected_text}', but got: {explanation}"
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule generates help text containing the
    /// expected string
    pub fn assert_help_contains(&self, code: &str, expected_text: &str) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations, but found none",
            self.id
        );

        let help = violations[0]
            .help
            .as_ref()
            .expect("Expected violation to have help text");

        assert!(
            help.contains(expected_text),
            "Expected help to contain '{expected_text}', but got: {help}"
        );
    }

    #[cfg(test)]
    #[track_caller]
    /// Test helper: assert that the rule generates a suggestion containing the
    /// expected string (deprecated: use assert_help_contains instead)
    pub fn assert_suggestion_contains(&self, code: &str, expected_text: &str) {
        self.assert_help_contains(code, expected_text)
    }
}
