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
    #[must_use] pub const fn as_str(self) -> &'static str {
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

    /// Get the rule ID
    #[must_use] pub const fn id(&self) -> &'static str {
        self.id
    }

    /// Get the rule category
    #[must_use] pub const fn category(&self) -> RuleCategory {
        self.category
    }

    /// Get the rule severity
    #[must_use] pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Get the rule description
    #[must_use] pub const fn description(&self) -> &'static str {
        self.description
    }
}