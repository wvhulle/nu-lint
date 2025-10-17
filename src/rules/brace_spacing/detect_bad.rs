#[cfg(test)]
mod tests {

    use crate::{context::LintContext, rule::Rule, rules::brace_spacing::BraceSpacing};

    #[test]
    fn test_bad_brace_spacing() {
        let rule = BraceSpacing;
        let bad = "{key: value}";

        LintContext::test_with_parsed_source(bad, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
        });
    }

    #[test]
    fn test_detect_record_without_spaces() {
        let rule = BraceSpacing;
        let bad_code = r#"let record = {name: "test", value: 42}"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect record braces without spaces"
            );
        });
    }

    #[test]
    fn test_detect_config_without_spaces() {
        let rule = BraceSpacing;
        let bad_code = r#"let config = {host: "localhost", port: 8080}"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect config braces without spaces"
            );
        });
    }

    #[test]
    fn test_detect_nested_without_spaces() {
        let rule = BraceSpacing;
        let bad_code = r#"let nested = {outer: {inner: "value"}}"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect nested braces without spaces"
            );
        });
    }
}
