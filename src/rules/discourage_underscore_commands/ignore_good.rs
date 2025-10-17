#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::discourage_underscore_commands::DiscourageUnderscoreCommands;
    use crate::parser::parse_source;
    use nu_protocol::engine::EngineState;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_hyphenated_command_not_flagged() {
        let rule = DiscourageUnderscoreCommands::new();

        let good_code = r"def my-command [param: string] {
    echo $param
}";

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes());
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
        let rule = DiscourageUnderscoreCommands::new();

        let good_code = r"def command [param: string] {
    echo $param
}";

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes());
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
