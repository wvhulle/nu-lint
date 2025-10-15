use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct AvoidMutableAccumulation;

impl AvoidMutableAccumulation {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AvoidMutableAccumulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for AvoidMutableAccumulation {
    fn id(&self) -> &str {
        "BP002"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Prefer functional pipelines over mutable list accumulation"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Look for mut var = [] followed by append pattern
        let pattern = Regex::new(r"mut\s+\w+\s*=\s*\[\s*\]").unwrap();

        context.violations_from_regex_if(&pattern, self.id(), self.severity(), |mat| {
            let var_text = mat.as_str();
            // Extract variable name
            let var_name = var_text.split_whitespace().nth(1)?;

            // Look for append pattern with this variable
            let append_pattern = format!(r"\${}.*\|\s*append", regex::escape(var_name));
            if Regex::new(&append_pattern)
                .unwrap()
                .is_match(context.source)
            {
                Some((
                    format!(
                        "Mutable list '{}' with append - consider using functional pipeline",
                        var_name
                    ),
                    Some("Use '$items | each { ... }' instead of mutable accumulation".to_string()),
                ))
            } else {
                None
            }
        })
    }
}
