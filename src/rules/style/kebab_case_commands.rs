use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct KebabCaseCommands {
    pattern: Regex,
}

impl KebabCaseCommands {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap(),
        }
    }

    fn is_valid_kebab_case(&self, name: &str) -> bool {
        self.pattern.is_match(name)
    }
}

impl Default for KebabCaseCommands {
    fn default() -> Self {
        Self::new()
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
        let def_pattern = Regex::new(r"\bdef\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\[").unwrap();

        def_pattern
            .captures_iter(context.source)
            .filter_map(|cap| {
                let cmd_match = cap.get(1)?;
                let cmd_name = cmd_match.as_str();

                if !self.is_valid_kebab_case(cmd_name) {
                    Some(Violation {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        message: format!(
                            "Command '{}' should use kebab-case naming convention",
                            cmd_name
                        ),
                        span: nu_protocol::Span::new(cmd_match.start(), cmd_match.end()),
                        suggestion: Some(format!(
                            "Consider renaming to: {}",
                            to_kebab_case(cmd_name)
                        )),
                        fix: None,
                        file: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' {
            result.push('-');
            prev_is_lower = false;
        } else if c.is_uppercase() {
            if i > 0 && prev_is_lower {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_lower = false;
        } else {
            result.push(c);
            prev_is_lower = c.is_lowercase();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("myCommand"), "my-command");
        assert_eq!(to_kebab_case("my_command"), "my-command");
        assert_eq!(to_kebab_case("my-command"), "my-command");
    }

    #[test]
    fn test_is_valid_kebab_case() {
        let rule = KebabCaseCommands::new();
        assert!(rule.is_valid_kebab_case("my-command"));
        assert!(rule.is_valid_kebab_case("command"));
        assert!(rule.is_valid_kebab_case("my-long-command"));
        assert!(!rule.is_valid_kebab_case("myCommand"));
        assert!(!rule.is_valid_kebab_case("my_command"));
    }
}
