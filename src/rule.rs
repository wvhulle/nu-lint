use crate::{
    context::LintContext,
    lint::{Severity, Violation},
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
            RuleCategory::Naming => "naming",
            RuleCategory::Formatting => "formatting",
            RuleCategory::Idioms => "idioms",
            RuleCategory::ErrorHandling => "error-handling",
            RuleCategory::CodeQuality => "code-quality",
            RuleCategory::Documentation => "documentation",
            RuleCategory::TypeSafety => "type-safety",
            RuleCategory::Performance => "performance",
        }
    }
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A concrete rule struct that wraps the check function
pub struct Rule {
    pub id: &'static str,
    pub category: RuleCategory,
    pub severity: Severity,
    pub description: &'static str,
    pub check: fn(&LintContext) -> Vec<Violation>,
}

impl Rule {
    /// Create a new rule
    pub const fn new(
        id: &'static str,
        category: RuleCategory,
        severity: Severity,
        description: &'static str,
        check: fn(&LintContext) -> Vec<Violation>,
    ) -> Self {
        Self {
            id,
            category,
            severity,
            description,
            check,
        }
    }

    /// Run the rule's check function on the given context
    #[must_use]
    pub fn check(&self, context: &LintContext) -> Vec<Violation> {
        (self.check)(context)
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    /// Test helper: assert that the rule finds violations in the given code
    pub fn assert_detects(&self, code: &str) {
        LintContext::test_with_parsed_source(code, |context| {
            let violations = self.check(&context);
            assert!(
                !violations.is_empty(),
                "Expected rule '{}' to detect violations in code, but found none",
                self.id
            );
        });
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    /// Test helper: assert that the rule finds no violations in the given code
    pub fn assert_ignores(&self, code: &str) {
        LintContext::test_with_parsed_source(code, |context| {
            let violations = self.check(&context);
            assert!(
                violations.is_empty(),
                "Expected rule '{}' to ignore code, but found {} violations",
                self.id,
                violations.len()
            );
        });
    }

    #[cfg(test)]
    #[allow(clippy::missing_panics_doc)]
    /// Test helper: assert that the rule finds at least the expected number of
    /// violations
    pub fn assert_violation_count(&self, code: &str, expected_min: usize) {
        LintContext::test_with_parsed_source(code, |context| {
            let violations = self.check(&context);
            assert!(
                violations.len() >= expected_min,
                "Expected rule '{}' to find at least {} violations, but found {}",
                self.id,
                expected_min,
                violations.len()
            );
        });
    }
}
