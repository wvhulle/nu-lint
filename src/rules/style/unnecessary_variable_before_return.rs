use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;

pub struct UnnecessaryVariableBeforeReturn;

impl UnnecessaryVariableBeforeReturn {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnnecessaryVariableBeforeReturn {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for UnnecessaryVariableBeforeReturn {
    fn id(&self) -> &str {
        "S009"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Variable assigned and immediately returned adds unnecessary verbosity"
    }

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
                        "Variable '{}' is assigned and immediately returned - consider returning the expression directly",
                        var_name1
                    ),
                    Some("Return the expression directly instead of assigning to a variable first".to_string()),
                ))
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnecessary_variable_detected() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let bad_code = r"def foo [] {
  let result = (some | pipeline)
  $result
}";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect unnecessary variable"
        );
    }

    #[test]
    fn test_variable_used_multiple_times_not_flagged() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let good_code = r"def foo [] {
  let result = (some | pipeline)
  print $result
  $result
}";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag variable used multiple times"
        );
    }

    #[test]
    fn test_direct_return_not_flagged() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let good_code = r#"def foo [] {
  some | pipeline
}"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag direct return"
        );
    }
}
