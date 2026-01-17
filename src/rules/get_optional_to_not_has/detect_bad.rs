use super::RULE;

#[test]
fn detect_get_o_is_empty() {
    let code = r#"$record | get -o key | is-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_get_optional_is_empty() {
    let code = r#"$record | get --optional key | is-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_get_i_is_empty() {
    let code = r#"$record | get -i key | is-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_variable_key() {
    let code = r#"$record | get -o $key | is-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_inline_record() {
    let code = r#"{a: 1, b: 2} | get -o c | is-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_with_pipeline_before_get() {
    let code = r#"$data | select field | get -o key | is-empty"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_inside_where_closure() {
    let code = r#"$targets | where {|t| $available | get -i $t | is-empty }"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_inside_each_closure() {
    let code = r#"$items | each {|it| $record | get -o $it | is-empty }"#;
    RULE.assert_detects(code);
}
