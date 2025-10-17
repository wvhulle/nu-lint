#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::Rule, rules::discourage_bare_ignore::DiscouragedBareIgnore,
    };

    #[test]
    fn test_bare_ignore_detected() {
        let rule = DiscouragedBareIgnore::new();
        let bad_code = r"
some | pipeline | each { |x| process $x } | ignore
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect bare ignore"
            );
        });
    }

    #[test]
    fn test_detect_bare_ignore_complex_pipeline() {
        let rule = DiscouragedBareIgnore::new();
        let bad_code = "some | pipeline | each { |x| process $x } | ignore";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect bare ignore in complex pipeline"
            );
        });
    }

    #[test]
    fn test_detect_bare_ignore_simple() {
        let rule = DiscouragedBareIgnore::new();
        let bad_code = "another | operation | ignore";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect bare ignore in simple operation"
            );
        });
    }
}
