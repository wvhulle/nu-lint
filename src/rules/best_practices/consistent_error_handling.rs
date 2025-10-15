use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct ConsistentErrorHandling;

impl ConsistentErrorHandling {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConsistentErrorHandling {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ConsistentErrorHandling {
    fn id(&self) -> &str {
        "BP005"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Check external command results consistently for better error handling"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Pattern: external command with complete but no exit code check
        // Matches: let result = (^command | complete) but no check for exit_code
        let complete_pattern =
            Regex::new(r"let\s+(\w+)\s*=\s*\([^)]*\^[^)]*\|\s*complete\s*\)").unwrap();
        let var_pattern = Regex::new(r"let\s+(\w+)").unwrap();

        context.violations_from_regex_if(&complete_pattern, self.id(), self.severity(), |mat| {
            let caps = var_pattern.captures(mat.as_str())?;
            let var_name = &caps[1];

            // Check if exit_code is checked within reasonable distance
            let remaining_source = &context.source[mat.end()..];
            let next_100_chars = &remaining_source[..remaining_source.len().min(200)];

            let exit_code_check = format!(r"\${}\s*\.\s*exit_code", regex::escape(var_name));

            if Regex::new(&exit_code_check).unwrap().is_match(next_100_chars) {
                None
            } else {
                Some((
                    format!(
                        "External command result '{}' stored but exit code not checked",
                        var_name
                    ),
                    Some("Check 'exit_code' field to handle command failures: if $result.exit_code != 0 { ... }".to_string()),
                ))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_exit_code_check() {
        let rule = ConsistentErrorHandling::new();

        let bad_code = r#"
let result = (^bluetoothctl info $mac | complete)
let output = $result.stdout
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect missing exit_code check"
        );
    }

    #[test]
    fn test_exit_code_checked() {
        let rule = ConsistentErrorHandling::new();

        let good_code = r#"
let result = (^bluetoothctl info $mac | complete)
if $result.exit_code != 0 {
    return
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag when exit_code is checked"
        );
    }

    #[test]
    fn test_no_complete_not_flagged() {
        let rule = ConsistentErrorHandling::new();

        let good_code = r#"
let result = (some | regular | pipeline)
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag non-external commands"
        );
    }
}
