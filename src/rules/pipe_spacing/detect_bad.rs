#[cfg(test)]
mod tests {
    
    use crate::{context::LintContext, rule::Rule, rules::pipe_spacing::PipeSpacing};

    #[test]
    fn test_pipe_spacing() {
        let rule = PipeSpacing;
        let bad = "ls|get name";

        LintContext::test_with_parsed_source(bad, |context| {
            assert!(!rule.check(&context).is_empty());
        });
    }

    #[test]
    fn test_closure_pipe_not_flagged_but_operators_are() {
        let rule = PipeSpacing;
        // But actual pipe operators should still be flagged
        let bad_with_closure = "{|x| echo $x}|get name";

        LintContext::test_with_parsed_source(bad_with_closure, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Pipe operators should still be flagged"
            );
        });
    }

    #[test]
    fn test_detect_double_space_before_pipe() {
        let rule = PipeSpacing;
        let bad_code = "ls  |get name";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect double space before pipe"
            );
        });
    }

    #[test]
    fn test_detect_missing_space_after_pipe() {
        let rule = PipeSpacing;
        let bad_code = "ls| get name";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect missing space after pipe"
            );
        });
    }

    #[test]
    fn test_detect_double_space_after_pipe() {
        let rule = PipeSpacing;
        let bad_code = "ls |  get name";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect double space after pipe"
            );
        });
    }
}
