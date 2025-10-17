#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::brace_spacing::BraceSpacing;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_bad_brace_spacing() {
        let rule = BraceSpacing::default();

        let bad = "{key: value}";
        let context = LintContext::test_from_source(bad);
        let violations = rule.check(&context);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_detect_record_without_spaces() {
        let rule = BraceSpacing::default();

        let bad_code = r#"let record = {name: "test", value: 42}"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect record braces without spaces"
        );
    }

    #[test]
    fn test_detect_config_without_spaces() {
        let rule = BraceSpacing::default();

        let bad_code = r#"let config = {host: "localhost", port: 8080}"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect config braces without spaces"
        );
    }

    #[test]
    fn test_detect_nested_without_spaces() {
        let rule = BraceSpacing::default();

        let bad_code = r#"let nested = {outer: {inner: "value"}}"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect nested braces without spaces"
        );
    }
}