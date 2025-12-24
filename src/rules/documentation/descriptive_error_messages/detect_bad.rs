use super::RULE;
use crate::log::instrument;

#[test]
fn test_detect_error_keyword_in_msg() {
    instrument();
    let bad_code = r#"
def process-file [file: string] {
    if not ($file | path exists) {
        error make { msg: "error" }
    }
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_failed_keyword_in_msg() {
    instrument();
    let bad_code = r#"
def convert-data [input] {
    if ($input | is-empty) {
        error make { msg: "failed" }
    }
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_vague_something_went_wrong_msg() {
    instrument();
    let bad_code = r#"
def validate [data] {
    error make { msg: "something went wrong" }
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_short_error_message() {
    instrument();
    let bad_code = r#"
def validate [data] {
    error make { msg: "bad input" }
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_error_prefix_pattern() {
    instrument();
    let bad_code = r#"
def validate [data] {
    error make { msg: "Error: x" }
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_print_stderr_vague() {
    instrument();
    let bad_code = r#"
def validate [data] {
    print -e "error"
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_print_stderr_short() {
    instrument();
    let bad_code = r#"
def validate [data] {
    print -e "failed"
}
"#;

    RULE.assert_detects(bad_code);
}
