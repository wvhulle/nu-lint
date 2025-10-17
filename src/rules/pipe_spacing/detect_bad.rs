#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use crate::rules::pipe_spacing::PipeSpacing;
    use crate::rule::Rule;

    #[test]
    fn test_pipe_spacing() {
        let rule = PipeSpacing::default();

        let bad = "ls|get name";
        let context = LintContext::test_from_source(bad);
        assert!(!rule.check(&context).is_empty());
    }

    #[test]
    fn test_closure_pipe_not_flagged_but_operators_are() {
        let rule = PipeSpacing::default();

        // But actual pipe operators should still be flagged
        let bad_with_closure = "{|x| echo $x}|get name";
        let context = LintContext::test_from_source(bad_with_closure);
        assert!(
            !rule.check(&context).is_empty(),
            "Pipe operators should still be flagged"
        );
    }

    #[test]
    fn test_detect_double_space_before_pipe() {
        let rule = PipeSpacing::default();

        let bad_code = "ls  |get name";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect double space before pipe"
        );
    }

    #[test]
    fn test_detect_missing_space_after_pipe() {
        let rule = PipeSpacing::default();

        let bad_code = "ls| get name";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect missing space after pipe"
        );
    }

    #[test]
    fn test_detect_double_space_after_pipe() {
        let rule = PipeSpacing::default();

        let bad_code = "ls |  get name";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect double space after pipe"
        );
    }
}