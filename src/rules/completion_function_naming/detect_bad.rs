#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::completion_function_naming::CompletionFunctionNaming;
    use crate::context::LintContext;
    use crate::rule::Rule;
    use crate::parser::parse_source;
    use nu_protocol::engine::EngineState;

    #[test]
    fn test_bad_completion_naming_detected() {
        let rule = CompletionFunctionNaming::new();

        let bad_code = r"def complete-branches [] { ^git branch }";
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
            "Should detect bad completion function naming"
        );
    }
}
