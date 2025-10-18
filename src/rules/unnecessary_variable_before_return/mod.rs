use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
};

pub struct UnnecessaryVariableBeforeReturn;

impl UnnecessaryVariableBeforeReturn {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnnecessaryVariableBeforeReturn {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleMetadata for UnnecessaryVariableBeforeReturn {
    fn id(&self) -> &'static str {
        "unnecessary_variable_before_return"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::CodeQuality
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Variable assigned and immediately returned adds unnecessary verbosity"
    }
}

impl RegexRule for UnnecessaryVariableBeforeReturn {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Pattern: let var = (...)\n$var (with optional whitespace)
        // Since regex doesn't support backreferences, we match and manually verify
        let pattern = Regex::new(r"let\s+(\w+)\s*=\s*\([^)]+\)\s*\n\s*\$(\w+)\s*(?:\n|$)").unwrap();

        context.violations_from_regex_if(&pattern, self.id(), self.severity(), |mat| {
            let caps = pattern.captures(mat.as_str())?;
            let var_name1 = &caps[1];
            let var_name2 = &caps[2];

            // Check if the variable name matches
            if var_name1 == var_name2 {
                Some((
                    format!(
                        "Variable '{var_name1}' is assigned and immediately returned - consider \
                         returning the expression directly"
                    ),
                    Some(
                        "Return the expression directly instead of assigning to a variable first"
                            .to_string(),
                    ),
                ))
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
