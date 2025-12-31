use super::RULE;
use crate::log::init_env_log;

#[test]
fn detects_long_single_line_list() {
    init_env_log();
    let code = r#"let items = ["very", "long", "list", "with", "many", "items", "that", "should", "be", "multiline"]"#;
    RULE.assert_count(code, 1);
}

#[test]
fn detects_list_exceeding_80_chars() {
    init_env_log();
    let code = r#"let data = ["item1", "item2", "item3", "item4", "item5", "item6", "item7", "item8", "item9"]"#;
    RULE.assert_count(code, 1);
}

#[test]
fn detects_nested_list() {
    init_env_log();
    let code = r#"let config = [["nested", "list"], ["second", "nested"]]"#;
    RULE.assert_count(code, 1);
}
