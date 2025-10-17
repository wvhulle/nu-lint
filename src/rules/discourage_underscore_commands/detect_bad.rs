#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::discourage_underscore_commands::DiscourageUnderscoreCommands;
    use crate::parser::parse_source;
    use nu_protocol::engine::EngineState;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_underscore_command_detected() {
        let rule = DiscourageUnderscoreCommands::new();

        let bad_code = r"def my_command [param: string] {
    echo $param
}";

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes());
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(
            !violations.is_empty(),
            "Should detect underscore in command name"
        );
        assert_eq!(violations[0].rule_id, "discourage_underscore_commands");
    }

    #[test]
    fn test_multiple_underscores_detected() {
        let rule = DiscourageUnderscoreCommands::new();

        let bad_code = r"def my_very_long_command_name [param: string] {
    echo $param
}";

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes());
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(
            !violations.is_empty(),
            "Should detect multiple underscores in command name"
        );
        assert_eq!(violations[0].rule_id, "discourage_underscore_commands");
    }
}
