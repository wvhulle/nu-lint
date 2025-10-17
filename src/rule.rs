use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    visitor::AstVisitor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Style,
    BestPractices,
    Performance,
    Documentation,
    TypeSafety,
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleCategory::Style => write!(f, "style"),
            RuleCategory::BestPractices => write!(f, "best-practices"),
            RuleCategory::Performance => write!(f, "performance"),
            RuleCategory::Documentation => write!(f, "documentation"),
            RuleCategory::TypeSafety => write!(f, "type-safety"),
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

    /// Create an AST visitor for this rule (for combined traversal
    /// optimization)
    fn create_visitor<'a>(&'a self, context: &'a LintContext<'a>) -> Box<dyn AstVisitor + 'a>;
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

    /// Create an AST visitor if this is an AST-based rule
    #[must_use]
    pub fn create_visitor<'a>(
        &'a self,
        context: &'a LintContext<'a>,
    ) -> Option<Box<dyn AstVisitor + 'a>> {
        match self {
            Rule::Regex(_) => None,
            Rule::Ast(rule) => Some(rule.create_visitor(context)),
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
