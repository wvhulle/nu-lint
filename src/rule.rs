use crate::{
    context::LintContext,
    lint::{Severity, Violation},
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

pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn category(&self) -> RuleCategory;
    fn severity(&self) -> Severity;
    fn description(&self) -> &str;

    fn check(&self, context: &LintContext) -> Vec<Violation>;
}
