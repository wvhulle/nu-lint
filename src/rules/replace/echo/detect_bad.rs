use super::rule;

#[test]
fn test_detect_echo_with_string() {
    let bad_code = r#"echo "hello world""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_echo() {
    let bad_code = r#"^echo "hello world""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_with_variable() {
    let bad_code = r"echo $value";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_in_pipeline() {
    let bad_code = r"echo $var | str upcase";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_with_multiple_args() {
    let bad_code = r"echo hello world";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_in_function() {
    let bad_code = r#"
def greet [name] {
    echo $"Hello ($name)"
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_in_closure() {
    let bad_code = r"
ls | each { |file|
    echo $file.name
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_echo_uses() {
    let bad_code = r#"
echo "first"
echo "second"
"#;
    rule().assert_count(bad_code, 2);
}
