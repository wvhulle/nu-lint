use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;

pub struct MissingTypeAnnotation;

impl MissingTypeAnnotation {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for MissingTypeAnnotation {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MissingTypeAnnotation {
    fn id(&self) -> &'static str {
        "missing_type_annotation"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::TypeSafety
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Parameters should have type annotations"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Look for def with parameters without type annotations
        // Pattern: def name [param1, param2] where param doesn't have :
        let def_pattern = Regex::new(r"def\s+[a-zA-Z_-][a-zA-Z0-9_-]*\s*\[([^\]]+)\]").unwrap();

        context.violations_from_regex_if(&def_pattern, self.id(), self.severity(), |mat| {
            let caps = def_pattern.captures(mat.as_str())?;
            let params_text = caps.get(1)?.as_str();

            // Split by comma and check each parameter
            for param in params_text.split(',') {
                let param = param.trim();
                // Skip if empty or already has type annotation (contains :)
                if !param.is_empty() && !param.contains(':') && !param.starts_with("--") {
                    // This is a parameter without type annotation
                    return Some((
                        format!("Parameter '{param}' is missing type annotation"),
                        Some(
                            "Add type annotation like 'param: string' or 'param: int'".to_string(),
                        ),
                    ));
                }
            }
            None
        })
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
#[cfg(test)]
mod generated_fix;