use super::RULE;

#[test]
fn detect_get_o_is_not_empty() {
    let code = r#"$record | get -o key | is-not-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_get_optional_is_not_empty() {
    let code = r#"$record | get --optional key | is-not-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_get_i_is_not_empty() {
    let code = r#"$record | get -i key | is-not-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_variable_key() {
    let code = r#"$record | get -o $key | is-not-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_inline_record() {
    let code = r#"{a: 1, b: 2} | get -o c | is-not-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_pipeline_before_get() {
    let code = r#"$data | select field | get -o key | is-not-empty"#;
    RULE.assert_detects(code);
}
