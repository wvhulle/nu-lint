use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct DiscourageUnderscoreCommands;

impl DiscourageUnderscoreCommands {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiscourageUnderscoreCommands {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for DiscourageUnderscoreCommands {
    fn id(&self) -> &str {
        "S012"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Command names should use hyphens instead of underscores for better readability"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Match: def command_name [...] or def "command_name" [...]
        let pattern = Regex::new(r#"def\s+(?:")?(\w+_\w+)(?:")?\s*\["#).unwrap();

        pattern
            .captures_iter(context.source)
            .map(|caps| {
                let command_name = &caps[1];
                let mat = caps.get(1).unwrap();
                let suggested_name = command_name.replace('_', "-");

                Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Command '{}' uses underscores - prefer hyphens for readability",
                        command_name
                    ),
                    span: nu_protocol::Span::new(mat.start(), mat.end()),
                    suggestion: Some(format!(
                        "Rename to '{}' following Nushell convention",
                        suggested_name
                    )),
                    fix: None,
                    file: None,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underscore_command_detected() {
        let rule = DiscourageUnderscoreCommands::new();

        let bad_code = r#"
def my_command [param: string] {
    echo $param
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect underscore in command name"
        );
    }

    #[test]
    fn test_hyphenated_command_not_flagged() {
        let rule = DiscourageUnderscoreCommands::new();

        let good_code = r#"
def my-command [param: string] {
    echo $param
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag hyphenated names"
        );
    }

    #[test]
    fn test_single_word_command_not_flagged() {
        let rule = DiscourageUnderscoreCommands::new();

        let good_code = r#"
def command [param: string] {
    echo $param
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag single-word names"
        );
    }
}
