use super::rule;

#[test]
fn detects_violations_for_long_list() {
    let code = r#"let items = ["very", "long", "list", "with", "many", "items", "that", "should", "be", "multiline"]"#;
    rule().assert_detects(code);
}
