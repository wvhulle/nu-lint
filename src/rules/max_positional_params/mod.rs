use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};

pub struct MaxPositionalParams {
    max_positional: usize,
}

impl MaxPositionalParams {
    #[must_use]
    pub fn new() -> Self {
        Self { max_positional: 2 }
    }
}

impl Default for MaxPositionalParams {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MaxPositionalParams {
    fn id(&self) -> &'static str {
        "max_positional_params"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Custom commands should have ≤ 2 positional parameters"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        context
            .new_user_functions()
            .filter_map(|(_, decl)| {
                let signature = decl.signature();

                // Count only positional parameters (not flags)
                let positional_count = signature.required_positional.len()
                    + signature.optional_positional.len()
                    + usize::from(signature.rest_positional.is_some());

                // Only create violation if count exceeds threshold
                (positional_count > self.max_positional).then(|| Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Command has {} positional parameters, should have ≤ {}",
                        positional_count, self.max_positional
                    ),
                    span: context.find_declaration_span(&signature.name),
                    suggestion: Some(
                        "Consider using named flags (--flag) for parameters beyond the first 2"
                            .to_string(),
                    ),
                    fix: None,
                    file: None,
                })
            })
            .collect()
    }
}


#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
#[cfg(test)]
mod generated_fix;