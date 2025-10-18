#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::AstRule, rules::prefer_is_not_empty::PreferIsNotEmpty,
    };

    #[test]
    fn test_not_is_empty_detected() {
        let rule = PreferIsNotEmpty;
        let bad_code = "if not ($list | is-empty) { echo 'has items' }";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect 'not ... is-empty'"
            );
        });
    }

    #[test]
    fn test_detect_not_is_empty_in_if() {
        let rule = PreferIsNotEmpty;
        let bad_code = r#"
if not ($list | is-empty) {
    print "has items"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect 'not is-empty' in if statement"
            );
        });
    }

    #[test]
    fn test_detect_not_is_empty_in_assignment() {
        let rule = PreferIsNotEmpty;
        let bad_code = "let has_data = not ($data | is-empty)";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect 'not is-empty' in assignment"
            );
        });
    }
}
