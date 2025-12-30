use super::RULE;

#[test]
fn fix_long_single_line_list() {
    let code = r#"let items = ["very", "long", "list", "with", "many", "items", "that", "should", "be", "multiline"]"#;
    let expected = r#"[
    "very"
    "long"
    "list"
    "with"
    "many"
    "items"
    "that"
    "should"
    "be"
    "multiline"
]"#;
    RULE.assert_replacement_contains(code, expected);
}

#[test]
fn fix_list_exceeding_80_chars() {
    let code = r#"let data = ["item1", "item2", "item3", "item4", "item5", "item6", "item7", "item8", "item9"]"#;
    let expected = r#"[
    "item1"
    "item2"
    "item3"
    "item4"
    "item5"
    "item6"
    "item7"
    "item8"
    "item9"
]"#;
    RULE.assert_replacement_contains(code, expected);
}

#[test]
fn fix_nested_list() {
    let code = r#"let config = [["nested", "list"], ["second", "nested"]]"#;
    let expected = r#"[
    ["nested", "list"]
    ["second", "nested"]
]"#;
    RULE.assert_replacement_contains(code, expected);
}
