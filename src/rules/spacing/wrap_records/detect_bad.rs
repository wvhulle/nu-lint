use super::RULE;
use crate::log::init_env_log;

#[test]
fn detects_long_single_line_record() {
    init_env_log();
    let code = r#"let config = {name: "very long name here", explanation: "very long description text", version: "1.0.0"}"#;
    RULE.assert_count(code, 1);
}

#[test]
fn detects_record_exceeding_80_chars() {
    init_env_log();
    let code = r#"let data = {key1: "value1", key2: "value2", key3: "value3", key4: "value4", key5: "value5x"}"#;
    RULE.assert_count(code, 1);
}

#[test]
fn detects_nested_record() {
    init_env_log();
    let code = r"let data = { id: 1, config: {nested: true} }";
    RULE.assert_count(code, 1);
}
