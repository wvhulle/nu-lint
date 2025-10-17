use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use heck::ToSnakeCase;
use regex::Regex;
use std::sync::OnceLock;

#[derive(Default)]
pub struct SnakeCaseVariables;

impl SnakeCaseVariables {
    fn snake_case_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap())
    }

    fn let_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"\blet\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap())
    }

    fn is_valid_snake_case(name: &str) -> bool {
        Self::snake_case_pattern().is_match(name)
    }
}

impl Rule for SnakeCaseVariables {
    fn id(&self) -> &'static str {
        "snake_case_variables"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Variables should use snake_case naming convention"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        Self::let_pattern()
            .captures_iter(context.source)
            .filter_map(|cap| {
                let var_match = cap.get(1)?;
                let var_name = var_match.as_str();

                (!Self::is_valid_snake_case(var_name)).then(|| Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Variable '{var_name}' should use snake_case naming convention"
                    ),
                    span: nu_protocol::Span::new(var_match.start(), var_match.end()),
                    suggestion: Some(format!(
                        "Consider renaming to: {}",
                        var_name.to_snake_case()
                    )),
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