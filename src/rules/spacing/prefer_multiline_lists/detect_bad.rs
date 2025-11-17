use super::rule;
use crate::log::instrument;

#[test]
fn detects_long_single_line_list() {
    instrument();
    let code = r#"let items = ["very", "long", "list", "with", "many", "items", "that", "should", "be", "multiline"]"#;
    rule().assert_count(code, 1);
}

#[test]
fn detects_list_exceeding_80_chars() {
    instrument();
    let code = r#"let data = ["item1", "item2", "item3", "item4", "item5", "item6", "item7", "item8", "item9"]"#;
    rule().assert_count(code, 1);
}

#[test]
fn detects_nested_list() {
    instrument();
    let code = r#"let config = [["nested", "list"], ["second", "nested"]]"#;
    rule().assert_count(code, 1);
}
