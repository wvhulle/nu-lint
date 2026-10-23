use super::RULE;

#[test]
fn ignore_has_operator() {
    let code = r#"$record has key"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_get_o_followed_by_other_command() {
    let code = r#"$record | get -o key | str length"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_get_without_optional_flag() {
    let code = r#"$record | get key | is-not-empty"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_is_not_empty_without_get() {
    let code = r#"$list | is-not-empty"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_get_o_with_is_empty() {
    // This is for the other rule (get_optional_to_not_has)
    let code = r#"$record | get -o key | is-empty"#;
    RULE.assert_ignores(code);
}
