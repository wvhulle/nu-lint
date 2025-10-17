#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::Rule,
        rules::descriptive_error_messages::DescriptiveErrorMessages,
    };

    #[test]
    fn test_detect_generic_error_message() {
        let rule = DescriptiveErrorMessages::new();
        let bad_code = r#"
def process-file [file: string] {
    if not ($file | path exists) {
        error make { msg: "error" }
    }
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect generic 'error' message"
            );
        });
    }

    #[test]
    fn test_generic_error_message_detected() {
        let source = r#"
def process [] {
    if $condition {
        error make { msg: "error" }
    }
}
"#;
        let rule = DescriptiveErrorMessages::new();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect generic error message"
            );
            assert_eq!(violations[0].rule_id, "descriptive_error_messages");
        });
    }

    #[test]
    fn test_failed_error_message_detected() {
        let source = r#"
def process [] {
    error make { msg: "failed" }
}
"#;
        let rule = DescriptiveErrorMessages::new();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect 'failed' as generic message"
            );
            assert_eq!(violations[0].rule_id, "descriptive_error_messages");
        });
    }

    #[test]
    fn test_detect_vague_failed_message() {
        let rule = DescriptiveErrorMessages::new();
        let bad_code = r#"
def convert-data [input] {
    if ($input | is-empty) {
        error make { msg: "failed" }
    }
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect vague 'failed' message"
            );
        });
    }

    #[test]
    fn test_detect_something_went_wrong_message() {
        let rule = DescriptiveErrorMessages::new();
        let bad_code = r#"
def validate [data] {
    error make { msg: "something went wrong" }
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect 'something went wrong' message"
            );
        });
    }
}
