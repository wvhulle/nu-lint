use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_print_exit_pattern() {
    let bad_code = r#"
def bad-error [] {
    print "Error occurred"
    exit 1
}
"#;
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_various_error_messages() {
    for msg in [
        "Error: File not found",
        "Failed to connect to server",
        "Cannot parse input",
        "Unable to access resource",
        "Missing configuration file",
        "Access denied",
        "Connection timeout",
    ] {
        let bad_code = format!(r#"print "{msg}"; exit 1"#);
        rule().assert_detects(&bad_code);
    }
}

#[test]
fn test_detect_print_exit_with_invalid_message() {
    instrument();
    let bad_code = r#"
if ($args | is-empty) {
    print "Invalid arguments provided"
    exit 1
}
"#;
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_nested_in_function() {
    let bad_code = r#"
def main [] {
    def helper [] {
        print "Error in helper function"
        exit 1
    }
    helper
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_in_closure() {
    let bad_code = r#"
let checker = { ||
    print "Error in closure"
    exit 1
}
do $checker
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_same_pipeline_variations() {
    rule().assert_detects(r#"def validate [] { print "Validation failed"; exit 1 }"#);
    rule().assert_detects(r#"let result = some_command; print "Command failed"; exit 2"#);
    let bad_code = r#"
def check [value: int] {
    if $value < 0 {
        print "Negative value not allowed"; exit 1
    }
}
"#;
    rule().assert_detects(bad_code);
}
