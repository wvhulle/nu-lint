#[cfg(test)]
mod tests {

    use crate::{context::LintContext, rule::AstRule, rules::brace_spacing::BraceSpacing};

    #[test]
    fn test_space_before_closure_params() {
        let rule = BraceSpacing;
        // According to style guide: "{ |el|" is incorrect, should be "{|el|"
        let bad = "[[status]; [UP]] | all { |el| $el.status == UP }";

        LintContext::test_with_parsed_source(bad, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect space before closure parameters"
            );
        });
    }

    #[test]
    fn test_inconsistent_spacing_space_after_only() {
        let rule = BraceSpacing;
        // { x} is inconsistent - space after but not before
        let bad_code = r#"let record = { name: "test"}"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect inconsistent brace spacing (space after but not before)"
            );
        });
    }

    #[test]
    fn test_inconsistent_spacing_space_before_only() {
        let rule = BraceSpacing;
        // {x } is inconsistent - space before but not after
        let bad_code = r#"let config = {host: "localhost" }"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect inconsistent brace spacing (space before but not after)"
            );
        });
    }
}
