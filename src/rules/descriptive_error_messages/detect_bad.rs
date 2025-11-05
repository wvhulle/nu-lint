use super::rule;

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

    rule().assert_detects(source);
}

#[test]
fn test_failed_error_message_detected() {
    let source = r#"
def process [] {
    error make { msg: "failed" }
}
"#;

    rule().assert_detects(source);
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
