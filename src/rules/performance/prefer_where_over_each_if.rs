use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferWhereOverEachIf;

impl PreferWhereOverEachIf {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferWhereOverEachIf {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferWhereOverEachIf {
    fn id(&self) -> &str {
        "P001"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Performance
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use 'where' for filtering instead of 'each' with 'if'"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Look for "each { |item| if $item..." pattern
        let pattern = Regex::new(r"each\s*\{\s*\|[^}]+if\s+\$").unwrap();

        context.violations_from_regex(
            &pattern,
            self.id(),
            self.severity(),
            "Consider using 'where' for filtering instead of 'each' with 'if'",
            Some("Use '$list | where <condition>' for better performance".to_string()),
        )
    }
}
