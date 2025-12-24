use super::RULE;

#[test]
fn test_detect_not_is_empty_in_if_statement() {
    let bad_code = r#"
if not ($list | is-empty) {
    print "has items"
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_not_is_empty_in_variable_assignment() {
    let bad_code = "let has_data = not ($data | is-empty)";

    RULE.assert_detects(bad_code);
}
