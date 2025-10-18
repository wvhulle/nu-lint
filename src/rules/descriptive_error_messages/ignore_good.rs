use super::rule;
use crate::LintContext;

#[test]
fn test_descriptive_error_message_not_flagged() {
    let source = r#"
def process [input: int] {
    if $input < 0 {
        error make { msg: "Input must be non-negative, got: " + ($input | into string) }
    }
}
"#;
    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);

        assert!(
            violations.is_empty(),
            "Should not flag descriptive error messages"
        );
    });
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
    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);

        assert!(
            violations.is_empty(),
            "Should not flag specific error messages"
        );
    });
}
