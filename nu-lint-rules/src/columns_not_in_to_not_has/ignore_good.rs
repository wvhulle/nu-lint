use super::RULE;

#[test]
fn ignore_not_has_operator() {
    let code = r#"$record not-has key"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_not_in_list() {
    let code = r#"$item not-in $list"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_in_columns() {
    // This is for the other rule (columns_in_to_has)
    let code = r#"$key in ($record | columns)"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_not_in_without_columns() {
    let code = r#"$key not-in ($record | values)"#;
    RULE.assert_ignores(code);
}
