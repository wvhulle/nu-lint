use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferMatchOverIfChain;

impl PreferMatchOverIfChain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferMatchOverIfChain {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferMatchOverIfChain {
    fn id(&self) -> &str {
        "BP007"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use 'match' for value-based branching instead of if-else-if chains"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Pattern: if $var == value { } else if $var == value { } else { }
        // Since backreferences aren't supported, capture both variables and check manually
        let if_chain_pattern =
            Regex::new(r"if\s+\$(\w+)\s*==\s*[^\{]+\{[^\}]*\}\s*else\s+if\s+\$(\w+)\s*==").unwrap();

        violations.extend(context.violations_from_regex_if(&if_chain_pattern, self.id(), self.severity(), |mat| {
            let caps = if_chain_pattern.captures(mat.as_str())?;
            let var_name1 = &caps[1];
            let var_name2 = &caps[2];

            // Check if it's the same variable being compared
            if var_name1 == var_name2 {
                Some((
                    format!(
                        "If-else-if chain comparing '{}' to different values - consider using 'match'",
                        var_name1
                    ),
                    Some("Use 'match $var { value1 => { ... }, value2 => { ... }, _ => { ... } }' for clearer value-based branching".to_string()),
                ))
            } else {
                None
            }
        }));

        // Also detect multiple else-if chains (3+ branches) even if variable changes
        let multiple_else_if =
            Regex::new(r"if\s+[^\{]+\{[^\}]*\}\s*else\s+if\s+[^\{]+\{[^\}]*\}\s*else\s+if")
                .unwrap();

        violations.extend(context.violations_from_regex_if(&multiple_else_if, self.id(), self.severity(), |mat| {
            // Check if we already reported this location
            let already_reported = violations
                .iter()
                .any(|v| v.span.start <= mat.start() && mat.start() <= v.span.end);

            if !already_reported {
                Some((
                    "Long if-else-if chain - consider using 'match' for clearer branching".to_string(),
                    Some("For multiple related conditions, 'match' provides clearer pattern matching".to_string()),
                ))
            } else {
                None
            }
        }));

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_variable_if_chain_detected() {
        let rule = PreferMatchOverIfChain::new();

        let bad_code = r#"
let color = if $scope == "wan" { "red" } else if $scope == "lan" { "yellow" } else { "green" }
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect if-else-if chain on same variable"
        );
    }

    #[test]
    fn test_long_else_if_chain_detected() {
        let rule = PreferMatchOverIfChain::new();

        let bad_code = r#"
if $x > 10 { "high" } else if $x > 5 { "medium" } else if $x > 0 { "low" } else { "zero" }
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect long if-else-if chains"
        );
    }

    #[test]
    fn test_simple_if_else_not_flagged() {
        let rule = PreferMatchOverIfChain::new();

        let good_code = r#"
if $condition { "yes" } else { "no" }
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag simple if-else"
        );
    }

    #[test]
    fn test_match_statement_not_flagged() {
        let rule = PreferMatchOverIfChain::new();

        let good_code = r#"
match $scope {
    "wan" => { "red" },
    "lan" => { "yellow" },
    _ => { "green" }
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag match statements"
        );
    }
}
