use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};

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
        let mut violations = Vec::new();

        for (_decl_id, decl) in context.new_user_functions() {
            let command_name = &decl.signature().name;

            // Check if command name contains underscores
            if command_name.contains('_') {
                let suggested_name = command_name.replace('_', "-");
                let span = context.find_declaration_span(command_name);

                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Command '{}' uses underscores - prefer hyphens for readability",
                        command_name
                    ),
                    span,
                    suggestion: Some(format!(
                        "Rename to '{}' following Nushell convention",
                        suggested_name
                    )),
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

    #[test]
    fn test_underscore_command_detected() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = DiscourageUnderscoreCommands::new();

        let bad_code = r#"def my_command [param: string] {
    echo $param
}"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert!(
            !rule.check(&context).is_empty(),
            "Should detect underscore in command name"
        );
    }

    #[test]
    fn test_hyphenated_command_not_flagged() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = DiscourageUnderscoreCommands::new();

        let good_code = r#"def my-command [param: string] {
    echo $param
}"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes()).unwrap();
        let context = LintContext {
            source: good_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag hyphenated names"
        );
    }

    #[test]
    fn test_single_word_command_not_flagged() {
        use crate::parser::parse_source;
        use nu_protocol::engine::EngineState;

        let rule = DiscourageUnderscoreCommands::new();

        let good_code = r#"def command [param: string] {
    echo $param
}"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes()).unwrap();
        let context = LintContext {
            source: good_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag single-word names"
        );
    }
}
