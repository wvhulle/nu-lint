use core::fmt::{self, Display, Formatter};

use crate::{
    context::LintContext,
    violation::{RuleViolation, Severity},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    /// Rules about identifier naming conventions (`snake_case`, kebab-case,
    /// etc.)
    Naming,
    /// Code layout and whitespace formatting rules
    Formatting,
    /// Nushell-specific best practices and preferred patterns
    Idioms,
    /// Error management and safety patterns
    ErrorHandling,
    /// General code cleanliness and maintainability
    CodeQuality,
    /// Documentation requirements and standards
    Documentation,
    /// Type annotations and type safety
    TypeSafety,
    /// Performance optimizations and efficient patterns
    Performance,
}

impl RuleCategory {
    /// Get the string representation of this category
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Naming => "naming",
            Self::Formatting => "formatting",
            Self::Idioms => "idioms",
            Self::ErrorHandling => "error-handling",
            Self::CodeQuality => "code-quality",
            Self::Documentation => "documentation",
            Self::TypeSafety => "type-safety",
            Self::Performance => "performance",
        }
    }
}

impl Display for RuleCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A concrete rule struct that wraps the check function
pub struct Rule {
    pub id: &'static str,
    pub category: RuleCategory,
    pub severity: Severity,
    pub description: &'static str,
    pub(crate) check: fn(&LintContext) -> Vec<RuleViolation>,
}

impl Rule {
    /// Create a new rule
    pub(crate) const fn new(
        id: &'static str,
        category: RuleCategory,
        severity: Severity,
        description: &'static str,
        check: fn(&LintContext) -> Vec<RuleViolation>,
    ) -> Self {
        Self {
            id,
            category,
            severity,
            description,
            check,
        }
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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
    #[allow(clippy::missing_panics_doc)]
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

        let replacement_text = &fix.replacements[0].new_text;
        assert!(
            replacement_text.contains(expected_text),
            "Expected fix replacement text to contain '{expected_text}', but got: \
             {replacement_text}"
        );
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    #[track_caller]
    /// Test helper: assert that the rule generates a fix with description
    /// containing the expected string
    pub fn assert_fix_description_contains(&self, code: &str, expected_text: &str) {
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

        let description = &fix.description;
        assert!(
            description.contains(expected_text),
            "Expected fix description to contain '{expected_text}', but got: {description}"
        );
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    #[track_caller]
    /// Test helper: assert that the rule generates a suggestion containing the
    /// expected string
    pub fn assert_suggestion_contains(&self, code: &str, expected_text: &str) {
        let violations =
            LintContext::test_with_parsed_source(code, |context| (self.check)(&context));
        assert!(
            !violations.is_empty(),
            "Expected rule '{}' to detect violations, but found none",
            self.id
        );

        let suggestion = violations[0]
            .suggestion
            .as_ref()
            .expect("Expected violation to have a suggestion");

        assert!(
            suggestion.contains(expected_text),
            "Expected suggestion to contain '{expected_text}', but got: {suggestion}"
        );
    }
}
