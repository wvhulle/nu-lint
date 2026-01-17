use super::RULE;

#[test]
fn test_ignore_error_make_usage() {
    let code = r#"
        error make { msg: "File not found" }
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_error_make_in_function() {
    let code = r#"
        def validate [input] {
            error make { msg: "Invalid input" }
        }
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_print_stderr_in_main() {
    let code = r#"
        def main [] {
            print --stderr "Fatal error"
            exit 1
        }
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_print_stderr_top_level() {
    let code = r#"
        print --stderr "Script error"
        exit 1
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_print_without_stderr() {
    let code = r#"
        def helper [] {
            print "Normal output"
        }
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_print_stdout_in_function() {
    let code = r#"
        def process [] {
            print "Processing..."
        }
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_error_make_with_label() {
    let code = r#"
        def validate [input] {
            error make {
                msg: "Invalid input"
                label: {
                    text: "here"
                    span: (metadata $input).span
                }
            }
        }
    "#;
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_main_without_try() {
    let code = r#"
        def main [file: string] {
            if not ($file | path exists) {
                print --stderr $"File not found: ($file)"
                exit 1
            }
        }
    "#;
    RULE.assert_ignores(code);
}
