use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferErrorMake;

impl PreferErrorMake {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferErrorMake {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferErrorMake {
    fn id(&self) -> &str {
        "BP001"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use 'error make' for custom errors instead of 'print' + 'exit'"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Look for print followed by exit pattern
        let pattern =
            Regex::new(r"(print\s+[^\n]+\s+exit\s+\d+)|(print\s+-e\s+[^\n]+\s+exit\s+\d+)")
                .unwrap();

        context.violations_from_regex(
            &pattern,
            self.id(),
            self.severity(),
            "Consider using 'error make' instead of 'print' + 'exit'",
            Some(
                "Use 'error make { msg: \"error message\" }' for better error handling".to_string(),
            ),
        )
    }
}
