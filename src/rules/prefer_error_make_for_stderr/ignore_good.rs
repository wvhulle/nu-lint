use super::rule;

#[test]
fn test_ignore_error_make_usage() {
    let code = r#"
        error make { msg: "File not found" }
    "#;
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_error_make_with_label() {
    let code = r#"
        error make {
            msg: "Invalid input"
            label: {
                text: "here"
                span: (metadata $input).span
            }
        }
    "#;
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_print_without_stderr() {
    let code = r#"
        print "Normal output"
        exit 1
    "#;
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_print_stderr_without_exit() {
    let code = r#"
        print --stderr "Warning message"
    "#;
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_exit_without_print() {
    let code = "exit 1";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_separate_statements() {
    let code = r#"
        print --stderr "Message"
        do-something
        exit 1
    "#;
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_print_stdout() {
    let code = r#"
        print "Success"
        exit 0
    "#;
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_error_make_with_help() {
    let code = r#"
        error make {
            msg: "Configuration error"
            help: "Check your config.toml file"
        }
    "#;
    rule().assert_ignores(code);
}
