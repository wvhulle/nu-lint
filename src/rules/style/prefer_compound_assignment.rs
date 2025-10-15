use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferCompoundAssignment;

impl PreferCompoundAssignment {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferCompoundAssignment {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferCompoundAssignment {
    fn id(&self) -> &str {
        "S008"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use compound assignment operators (+=, -=, etc.) for clarity"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Pattern: $var = $var op value
        // Since regex doesn't support backreferences, we need to find all patterns and manually check
        let pattern = Regex::new(r"\$(\w+)\s*=\s*\$(\w+)\s*([+\-*/])\s*").unwrap();

        context.violations_from_regex_if(&pattern, self.id(), self.severity(), |mat| {
            let caps = pattern.captures(mat.as_str())?;
            let var_name1 = &caps[1];
            let var_name2 = &caps[2];

            // Check if both variable names are the same
            if var_name1 == var_name2 {
                let operator = &caps[3];
                let compound_op = format!("{}=", operator);

                Some((
                    format!(
                        "Use compound assignment: ${} {} instead of ${} = ${} {}",
                        var_name1, compound_op, var_name1, var_name1, operator
                    ),
                    Some(format!("Replace with: ${} {}", var_name1, compound_op)),
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
    fn test_addition_assignment_detected() {
        let rule = PreferCompoundAssignment::new();

        let bad_code = "$count = $count + 1";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect $x = $x + 1"
        );
    }

    #[test]
    fn test_subtraction_assignment_detected() {
        let rule = PreferCompoundAssignment::new();

        let bad_code = "$value = $value - 5";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect $x = $x - 5"
        );
    }

    #[test]
    fn test_compound_assignment_not_flagged() {
        let rule = PreferCompoundAssignment::new();

        let good_code = "$count += 1";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag compound assignment"
        );
    }

    #[test]
    fn test_different_variables_not_flagged() {
        let rule = PreferCompoundAssignment::new();

        let good_code = "$x = $y + 1";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag different variables"
        );
    }
}
