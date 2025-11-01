use super::rule;
use crate::clean_log::log;
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
fn test_detect_print_exit_with_error_message() {
    let bad_code = r#"
def process-file [path: string] {
    if not ($path | path exists) {
        print "Error: File not found"
        exit 1
    }
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_exit_with_failed_message() {
    let bad_code = r#"
print "Failed to connect to server"
exit 2
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_exit_with_cannot_message() {
    let bad_code = r#"
def validate [input] {
    print "Cannot parse input"
    exit 1
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_exit_with_invalid_message() {
    log();
    let bad_code = r#"
if ($args | is-empty) {
    print "Invalid arguments provided"
    exit 1
}
"#;
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_print_exit_with_unable_message() {
    let bad_code = r#"
print "Unable to access resource"
exit 3
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_exit_with_missing_message() {
    let bad_code = r#"
def check-config [] {
    print "Missing configuration file"
    exit 1
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_exit_with_denied_message() {
    let bad_code = r#"
print "Access denied"
exit 1
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_exit_with_timeout_message() {
    let bad_code = r#"
print "Connection timeout"
exit 1
"#;
    rule().assert_detects(bad_code);
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
