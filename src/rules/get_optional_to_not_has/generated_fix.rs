use super::RULE;

#[test]
fn fix_simple_case() {
    let code = r#"$record | get -o key | is-empty"#;
    RULE.assert_fixed_contains(code, "$record not-has key");
}

#[test]
fn fix_with_variable_key() {
    let code = r#"$record | get -o $key | is-empty"#;
    RULE.assert_fixed_contains(code, "$record not-has $key");
}

#[test]
fn fix_with_inline_record() {
    let code = r#"{a: 1, b: 2} | get -o c | is-empty"#;
    RULE.assert_fixed_contains(code, "{a: 1, b: 2} not-has c");
}

#[test]
fn fix_explanation() {
    let code = r#"$record | get -o key | is-empty"#;
    RULE.assert_fix_explanation_contains(code, "not-has");
}
