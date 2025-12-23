use super::rule;

#[test]
fn test_detect_print_or_echo_without_prefix() {
    for (cmd, msg) in [("print", "Hello, World!"), ("echo", "Starting process")] {
        let bad_code = format!(r#"{cmd} "{msg}""#);
        rule().assert_detects(&bad_code);
    }
}

#[test]
fn test_detect_isolated_print_without_prefix() {
    let bad_code = r#"
let x = 1
print "Starting task"
let y = 2
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_or_echo_in_function() {
    for (cmd, func, msg) in [
        ("print", "deploy", "Deploying application"),
        ("echo", "backup", "Backing up files"),
    ] {
        let bad_code = format!(
            r#"
def {func} [] {{
    {cmd} "{msg}"
}}
"#
        );
        rule().assert_detects(&bad_code);
    }
}

#[test]
fn test_detect_print_after_semicolon() {
    let bad_code = r#"let x = 5; print "Value set""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_with_variable() {
    let bad_code = r#"print $"Processing {$file}""#;
    rule().assert_detects(bad_code);
}
