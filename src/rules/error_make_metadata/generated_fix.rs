use super::rule;
use crate::LintContext;

#[test]
fn test_missing_both_label_and_help_includes_actual_msg() {
    let bad_code = r#"
def validate [input: string] {
    error make { msg: "Input cannot be empty" }
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 1);

        let violation = &violations[0];
        assert_eq!(violation.message, "error make call is missing metadata fields: label, help");

        let suggestion = violation.suggestion.as_ref().unwrap();
        assert!(suggestion.contains("Add 'label' and 'help' fields"));
        assert!(suggestion.contains("Input cannot be empty"), "Should include actual msg");
        assert!(suggestion.contains("(metadata $input).span"), "Should reference function parameter");
        assert!(suggestion.contains("label:"));
        assert!(suggestion.contains("help:"));
    });
}

#[test]
fn test_missing_help_only_shows_existing_msg() {
    let bad_code = r#"
def process [data: string] {
    error make {
        msg: "Invalid data"
        label: { text: "here" }
    }
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 1);

        let violation = &violations[0];
        assert_eq!(violation.message, "error make call is missing metadata fields: help");
        assert!(!violation.message.contains("label"), "Should only mention missing help field");

        let suggestion = violation.suggestion.as_ref().unwrap();
        assert!(suggestion.contains("Add 'help' field"), "Should reference help field");
        assert!(suggestion.contains("Invalid data"), "Should include actual msg");
        assert!(suggestion.contains("help:"));
    });
}

#[test]
fn test_missing_label_only_references_parameter() {
    let bad_code = r#"
def check [value: int] {
    error make {
        msg: "Value error"
        help: "Fix the value"
    }
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 1);

        let violation = &violations[0];
        assert_eq!(violation.message, "error make call is missing metadata fields: label");
        assert!(!violation.message.contains("help"), "Should only mention missing label field");

        let suggestion = violation.suggestion.as_ref().unwrap();
        assert!(suggestion.contains("Add 'label' field"), "Should reference label field");
        assert!(suggestion.contains("Value error"), "Should include actual msg");
        assert!(suggestion.contains("(metadata $value).span"), "Should reference function parameter");
        assert!(suggestion.contains("label:"));
    });
}

#[test]
fn test_truncates_long_messages() {
    let bad_code = r#"
def test [arg: string] {
    error make { msg: "This is a very long error message that should be truncated in suggestions" }
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 1);

        let suggestion = violations[0].suggestion.as_ref().unwrap();
        assert!(suggestion.contains("..."), "Long messages should be truncated");
    });
}
