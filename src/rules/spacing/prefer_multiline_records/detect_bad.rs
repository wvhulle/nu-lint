use super::rule;
use crate::log::instrument;

#[test]
fn detects_long_single_line_record() {
    instrument();
    let code = r#"let config = {name: "very long name here", explanation: "very long description text", version: "1.0.0"}"#;
    rule().assert_count(code, 1);
}

#[test]
fn detects_record_exceeding_80_chars() {
    instrument();
    let code = r#"let data = {key1: "value1", key2: "value2", key3: "value3", key4: "value4", key5: "value5x"}"#;
    rule().assert_count(code, 1);
}

#[test]
fn detects_nested_record() {
    instrument();
    let code = r"let data = { id: 1, config: {nested: true} }";
    rule().assert_count(code, 1);
}
