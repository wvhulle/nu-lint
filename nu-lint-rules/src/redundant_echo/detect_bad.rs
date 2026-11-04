use super::RULE;

#[test]
fn test_detect_echo_with_string() {
    let bad_code = r#"echo "hello world""#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_echo_with_variable() {
    let bad_code = r"echo $value";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_echo_in_pipeline() {
    let bad_code = r"echo $var | str upcase";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_echo_with_multiple_args() {
    let bad_code = r"echo hello world";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_echo_in_function() {
    let bad_code = r#"
def greet [name] {
    echo $"Hello ($name)"
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_echo_in_closure() {
    let bad_code = r"
ls | each { |file|
    echo $file.name
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_echo_uses() {
    let bad_code = r#"
echo "first"
echo "second"
"#;
    RULE.assert_count(bad_code, 2);
}

#[test]
fn detect_echo_inside_attribute_argument_block() {
    let bad_code = r#"
@example "greet" { echo hi }
def main [] { 1 }
"#;
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_aliased_echo_call() {
    let bad_code = "
alias say = echo
say hello
";
    RULE.assert_count(bad_code, 2);
}
