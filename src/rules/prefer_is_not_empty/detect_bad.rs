#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use crate::rules::prefer_is_not_empty::PreferIsNotEmpty;
    use crate::rule::Rule;

    #[test]
    fn test_not_is_empty_detected() {
        let rule = PreferIsNotEmpty::default();

        let bad_code = "if not ($list | is-empty) { echo 'has items' }";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect 'not ... is-empty'"
        );
    }

    #[test]
    fn test_detect_not_is_empty_in_if() {
        let rule = PreferIsNotEmpty::default();

        let bad_code = r#"
if not ($list | is-empty) {
    print "has items"
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect 'not is-empty' in if statement"
        );
    }

    #[test]
    fn test_detect_not_is_empty_in_assignment() {
        let rule = PreferIsNotEmpty::default();

        let bad_code = "let has_data = not ($data | is-empty)";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect 'not is-empty' in assignment"
        );
    }
}