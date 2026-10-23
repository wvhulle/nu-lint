use super::RULE;

#[test]
fn detect_key_in_columns() {
    let code = r#""key" in ($record | columns)"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_variable_key_in_columns() {
    let code = r#"$key in ($record | columns)"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_inline_record() {
    let code = r#""a" in ({a: 1, b: 2} | columns)"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_pipeline_in_record() {
    let code = r#"$key in ($data | select field | columns)"#;
    RULE.assert_detects(code);
}
