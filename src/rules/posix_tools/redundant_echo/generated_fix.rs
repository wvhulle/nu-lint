use super::RULE;

#[test]
fn test_fix_for_string_literal() {
    let bad_code = r#"echo "hello world""#;
    let expected = r#""hello world""#;

    RULE.assert_detects(bad_code);
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_for_variable() {
    let bad_code = r"echo $value";
    let expected = r"$value";

    RULE.assert_detects(bad_code);
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_for_pipeline() {
    let bad_code = r"echo $var | str upcase";
    let expected = r"$var | str upcase";

    RULE.assert_detects(bad_code);
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_for_external_echo() {
    let bad_code = r#"^echo "test""#;
    let expected = r#""test""#;

    RULE.assert_detects(bad_code);
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_for_multiple_arguments() {
    let bad_code = r"echo hello world test";
    let expected = r"hello world test";

    RULE.assert_detects(bad_code);
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_description() {
    let bad_code = r#"echo "result""#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_multiple_violations_each_have_fix() {
    let bad_code = r#"
echo "first"
echo $var
^echo "third"
"#;

    RULE.assert_count(bad_code, 3);
}
