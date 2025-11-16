use super::rule;

#[test]
fn test_fix_for_string_literal() {
    let bad_code = r#"echo "hello world""#;
    let expected = r#""hello world""#;

    rule().assert_detects(bad_code);
    rule().assert_fix(bad_code, expected);
}

#[test]
fn test_fix_for_variable() {
    let bad_code = r"echo $value";
    let expected = r"$value";

    rule().assert_detects(bad_code);
    rule().assert_fix(bad_code, expected);
}

#[test]
fn test_fix_for_pipeline() {
    let bad_code = r"echo $var | str upcase";
    let expected = r"$var";

    rule().assert_detects(bad_code);
    rule().assert_fix(bad_code, expected);
}

#[test]
fn test_fix_for_external_echo() {
    let bad_code = r#"^echo "test""#;
    let expected = r#""test""#;

    rule().assert_detects(bad_code);
    rule().assert_fix(bad_code, expected);
}

#[test]
fn test_fix_for_multiple_arguments() {
    let bad_code = r"echo hello world test";
    let expected = r"hello world test";

    rule().assert_detects(bad_code);
    rule().assert_fix(bad_code, expected);
}

#[test]
fn test_fix_description() {
    let bad_code = r#"echo "result""#;

    rule().assert_detects(bad_code);
    rule().assert_fix_explanation_contains(bad_code, "echo");
    rule().assert_fix_explanation_contains(bad_code, "result");
}

#[test]
fn test_multiple_violations_each_have_fix() {
    let bad_code = r#"
echo "first"
echo $var
^echo "third"
"#;

    rule().assert_violation_count_exact(bad_code, 3);
}
