use super::rule;
use crate::LintContext;

#[test]
fn test_detect_generic_error_message() {
    let bad_code = r#"
def process-file [file: string] {
    if not ($file | path exists) {
        error make { msg: "error" }
    }
}
"#;

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
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

    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);
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

    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);
        assert!(
            !violations.is_empty(),
            "Should detect 'failed' as generic message"
        );
        assert_eq!(violations[0].rule_id, "descriptive_error_messages");
    });
}

#[test]
fn test_detect_vague_failed_message() {
    let bad_code = r#"
def convert-data [input] {
    if ($input | is-empty) {
        error make { msg: "failed" }
    }
}
"#;

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_something_went_wrong_message() {
    let bad_code = r#"
def validate [data] {
    error make { msg: "something went wrong" }
}
"#;

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}
