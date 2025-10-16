use crate::case_conversion::to_kebab_case;
use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Default)]
pub struct KebabCaseCommands;

impl KebabCaseCommands {
    fn kebab_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap())
    }

    fn is_valid_kebab_case(name: &str) -> bool {
        Self::kebab_pattern().is_match(name)
    }
}

impl Rule for KebabCaseCommands {
    fn id(&self) -> &str {
        "S002"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Custom commands should use kebab-case naming convention"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (_decl_id, decl) in context.new_user_functions() {
            let cmd_name = &decl.signature().name;

            if !Self::is_valid_kebab_case(cmd_name) {
                let span = context.find_declaration_span(cmd_name);
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Command '{}' should use kebab-case naming convention",
                        cmd_name
                    ),
                    span,
                    suggestion: Some(format!("Consider renaming to: {}", to_kebab_case(cmd_name))),
                    fix: None,
                    file: None,
                });
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_conversion::to_kebab_case;

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("myCommand"), "my-command");
        assert_eq!(to_kebab_case("my_command"), "my-command");
        assert_eq!(to_kebab_case("my-command"), "my-command");
    }

    #[test]
    fn test_is_valid_kebab_case() {
        assert!(KebabCaseCommands::is_valid_kebab_case("my-command"));
        assert!(KebabCaseCommands::is_valid_kebab_case("command"));
        assert!(KebabCaseCommands::is_valid_kebab_case("my-long-command"));
        assert!(!KebabCaseCommands::is_valid_kebab_case("myCommand"));
        assert!(!KebabCaseCommands::is_valid_kebab_case("my_command"));
    }
}
