use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Default)]
pub struct PreferErrorMake;

impl PreferErrorMake {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"print\s+(?:-e\s+)?([^\n]+)\s*(?:;|\n)\s*exit\s+(\d+)").unwrap()
        })
    }
}

impl Rule for PreferErrorMake {
    fn id(&self) -> &'static str {
        "BP001"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Use 'error make' for custom errors instead of 'print' + 'exit'"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let pattern = Self::pattern();

        context.violations_from_regex_if(pattern, self.id(), self.severity(), |mat| {
            if let Some(caps) = pattern.captures(mat.as_str()) {
                let message = &caps[1].trim_matches('"').trim_matches('\'');
                let exit_code: i32 = caps[2].parse().unwrap_or(1);

                // Only suggest error make for actual error cases
                if Self::looks_like_error(message, exit_code) {
                    Some((
                        "Consider using 'error make' instead of 'print' + 'exit' for error conditions".to_string(),
                        Some("Use 'error make { msg: \"error message\" }' for better error handling".to_string()),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

impl PreferErrorMake {
    fn looks_like_error(message: &str, exit_code: i32) -> bool {
        let message_lower = message.to_lowercase();
        let error_indicators = [
            "error",
            "failed",
            "cannot",
            "unable",
            "invalid",
            "not found",
            "missing",
            "denied",
            "forbidden",
            "unauthorized",
            "timeout",
            "connection",
            "network",
            "unreachable",
        ];

        // Non-zero exit codes with error-like messages are likely errors
        exit_code != 0
            && error_indicators
                .iter()
                .any(|indicator| message_lower.contains(indicator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_print_exit_flagged() {
        let rule = PreferErrorMake::new();

        let error_code = r#"
print "Error: cannot connect to server"
exit 1
"#;
        let context = LintContext::test_from_source(error_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should flag error print + exit"
        );
    }

    #[test]
    fn test_informational_print_exit_not_flagged() {
        let rule = PreferErrorMake::new();

        let info_code = r#"
print "Website is not reachable - please try again later"
exit 1
"#;
        let context = LintContext::test_from_source(info_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag informational messages"
        );
    }

    #[test]
    fn test_success_exit_not_flagged() {
        let rule = PreferErrorMake::new();

        let success_code = r#"
print "Operation completed successfully"
exit 0
"#;
        let context = LintContext::test_from_source(success_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag success exit codes"
        );
    }

    #[test]
    fn test_various_error_patterns_flagged() {
        let rule = PreferErrorMake::new();

        let test_cases = [
            r#"print "Failed to download file"; exit 1"#,
            r#"print "Cannot access database"; exit 2"#,
            r#"print "Invalid configuration file"; exit 1"#,
            r#"print "File not found"; exit 1"#,
        ];

        for case in test_cases {
            let context = LintContext::test_from_source(case);
            assert!(
                !rule.check(&context).is_empty(),
                "Should flag error case: {}",
                case
            );
        }
    }

    #[test]
    fn test_non_error_patterns_not_flagged() {
        let rule = PreferErrorMake::new();

        let test_cases = [
            r#"print "Processing complete"; exit 0"#,
            r#"print "Thank you for using our tool"; exit 0"#,
            r#"print "Configuration saved"; exit 0"#,
        ];

        for case in test_cases {
            let context = LintContext::test_from_source(case);
            assert_eq!(
                rule.check(&context).len(),
                0,
                "Should not flag non-error case: {}",
                case
            );
        }
    }
}
