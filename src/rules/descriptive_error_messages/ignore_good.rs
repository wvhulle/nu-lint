#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::descriptive_error_messages::DescriptiveErrorMessages;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_descriptive_error_message_not_flagged() {
        let source = r#"
def process [input: int] {
    if $input < 0 {
        error make { msg: "Input must be non-negative, got: " + ($input | into string) }
    }
}
"#;
        let rule = DescriptiveErrorMessages::new();
        let context = LintContext::test_from_source(source);
        let violations = rule.check(&context);

        assert!(
            violations.is_empty(),
            "Should not flag descriptive error messages"
        );
    }

    #[test]
    fn test_specific_error_message_not_flagged() {
        let source = r#"
def validate_file [path: string] {
    if not ($path | path exists) {
        error make { msg: $"File not found: ($path)" }
    }
}
"#;
        let rule = DescriptiveErrorMessages::new();
        let context = LintContext::test_from_source(source);
        let violations = rule.check(&context);

        assert!(
            violations.is_empty(),
            "Should not flag specific error messages"
        );
    }
}
