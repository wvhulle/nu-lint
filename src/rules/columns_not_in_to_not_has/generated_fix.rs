use super::RULE;

#[test]
fn fix_simple_case() {
    let code = r#""key" not-in ($record | columns)"#;
    RULE.assert_fixed_contains(code, "$record not-has \"key\"");
}

#[test]
fn fix_with_variable_key() {
    let code = r#"$key not-in ($record | columns)"#;
    RULE.assert_fixed_contains(code, "$record not-has $key");
}

#[test]
fn fix_explanation() {
    let code = r#"$key not-in ($record | columns)"#;
    RULE.assert_fix_explanation_contains(code, "not-has");
}
