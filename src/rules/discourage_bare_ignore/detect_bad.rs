#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use crate::rules::discourage_bare_ignore::DiscouragedBareIgnore;
    use crate::rule::Rule;

    #[test]
    fn test_bare_ignore_detected() {
        let rule = DiscouragedBareIgnore::new();

        let bad_code = r"
some | pipeline | each { |x| process $x } | ignore
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect bare ignore"
        );
    }

    #[test]
    fn test_detect_bare_ignore_complex_pipeline() {
        let rule = DiscouragedBareIgnore::new();

        let bad_code = "some | pipeline | each { |x| process $x } | ignore";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect bare ignore in complex pipeline"
        );
    }

    #[test]
    fn test_detect_bare_ignore_simple() {
        let rule = DiscouragedBareIgnore::new();

        let bad_code = "another | operation | ignore";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect bare ignore in simple operation"
        );
    }
}