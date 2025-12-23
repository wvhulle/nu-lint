use super::rule;

#[test]
fn test_detect_print_stderr_exit_pattern() {
    let code = r#"
        print --stderr "File not found"
        exit 1
    "#;
    rule().assert_count(code, 1);
}

#[test]
fn test_detect_print_stderr_with_variable() {
    let code = r#"
        let msg = "Connection failed"
        print --stderr $msg
        exit 1
    "#;
    rule().assert_detects(code);
}

#[test]
fn test_detect_in_function() {
    let code = r#"
        def check-file [path: string] {
            if not ($path | path exists) {
                print --stderr "File does not exist"
                exit 1
            }
        }
    "#;
    rule().assert_detects(code);
}

#[test]
fn test_detect_in_closure() {
    let code = r#"
        let validator = {|file|
            print --stderr "Invalid file"
            exit 1
        }
    "#;
    rule().assert_detects(code);
}

#[test]
fn test_detect_with_zero_exit_code() {
    let code = r#"
        print --stderr "Warning message"
        exit 0
    "#;
    rule().assert_detects(code);
}

#[test]
fn test_detect_multiple_occurrences() {
    let code = r#"
        if $condition {
            print --stderr "Error 1"
            exit 1
        } else {
            print --stderr "Error 2"
            exit 2
        }
    "#;
    rule().assert_count(code, 2);
}

#[test]
fn test_detect_with_interpolation() {
    let code = r#"
        let file = "test.txt"
        print --stderr $"Could not open ($file)"
        exit 1
    "#;
    rule().assert_detects(code);
}

#[test]
fn test_detect_in_nested_block() {
    let code = r#"
        def main [] {
            if true {
                print --stderr "Nested error"
                exit 1
            }
        }
    "#;
    rule().assert_detects(code);
}

#[test]
fn test_detect_with_no_exit_code() {
    let code = r#"
        print --stderr "Default exit code"
        exit
    "#;
    rule().assert_detects(code);
}
