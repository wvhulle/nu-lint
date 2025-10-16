use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use heck::ToKebabCase;
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
    fn id(&self) -> &'static str {
        "S002"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Custom commands should use kebab-case naming convention"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        context
            .new_user_functions()
            .filter_map(|(_decl_id, decl)| {
                let cmd_name = &decl.signature().name;
                (!Self::is_valid_kebab_case(cmd_name)).then(|| Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Command '{cmd_name}' should use kebab-case naming convention"
                    ),
                    span: context.find_declaration_span(cmd_name),
                    suggestion: Some(format!(
                        "Consider renaming to: {}",
                        cmd_name.to_kebab_case()
                    )),
                    fix: None,
                    file: None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heck::ToKebabCase;

    #[test]
    fn test_to_kebab_case() {
        assert_eq!("myCommand".to_kebab_case(), "my-command");
        assert_eq!("my_command".to_kebab_case(), "my-command");
        assert_eq!("my-command".to_kebab_case(), "my-command");
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
