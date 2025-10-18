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

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleCategory::Naming => write!(f, "naming"),
            RuleCategory::Formatting => write!(f, "formatting"),
            RuleCategory::Idioms => write!(f, "idioms"),
            RuleCategory::ErrorHandling => write!(f, "error-handling"),
            RuleCategory::CodeQuality => write!(f, "code-quality"),
            RuleCategory::Documentation => write!(f, "documentation"),
            RuleCategory::TypeSafety => write!(f, "type-safety"),
            RuleCategory::Performance => write!(f, "performance"),
        }
    }
}

/// Common metadata for all rules
pub trait RuleMetadata: Send + Sync {
    fn id(&self) -> &str;
    fn category(&self) -> RuleCategory;
    fn severity(&self) -> Severity;
    fn description(&self) -> &str;
}

/// Rules that check code using regex patterns on source text
pub trait RegexRule: RuleMetadata {
    fn check(&self, context: &LintContext) -> Vec<Violation>;
}

/// Rules that check code using AST traversal
pub trait AstRule: RuleMetadata {
    fn check(&self, context: &LintContext) -> Vec<Violation>;
}

/// Type-safe wrapper for different rule implementations
pub enum Rule {
    Regex(Box<dyn RegexRule>),
    Ast(Box<dyn AstRule>),
}

impl Rule {
    /// Check this rule against the given context
    #[must_use]
    pub fn check(&self, context: &LintContext) -> Vec<Violation> {
        match self {
            Rule::Regex(rule) => rule.check(context),
            Rule::Ast(rule) => rule.check(context),
        }
    }

    /// Check if this is an AST-based rule
    #[must_use]
    pub fn is_ast_rule(&self) -> bool {
        matches!(self, Rule::Ast(_))
    }
}

impl RuleMetadata for Rule {
    fn id(&self) -> &str {
        match self {
            Rule::Regex(rule) => rule.id(),
            Rule::Ast(rule) => rule.id(),
        }
    }

    fn category(&self) -> RuleCategory {
        match self {
            Rule::Regex(rule) => rule.category(),
            Rule::Ast(rule) => rule.category(),
        }
    }

    fn severity(&self) -> Severity {
        match self {
            Rule::Regex(rule) => rule.severity(),
            Rule::Ast(rule) => rule.severity(),
        }
    }

    fn description(&self) -> &str {
        match self {
            Rule::Regex(rule) => rule.description(),
            Rule::Ast(rule) => rule.description(),
        }
    }
}
