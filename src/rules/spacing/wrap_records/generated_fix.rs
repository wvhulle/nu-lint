use super::RULE;

#[test]
fn fix_long_single_line_record() {
    let code = r#"let config = {name: "very long name here", explanation: "very long description text", version: "1.0.0"}"#;
    let expected = r#"{
    name: "very long name here"
    explanation: "very long description text"
    version: "1.0.0"
}"#;
    RULE.assert_replacement_contains(code, expected);
}

#[test]
fn fix_record_exceeding_80_chars() {
    let code = r#"let data = {key1: "value1", key2: "value2", key3: "value3", key4: "value4", key5: "value5x"}"#;
    let expected = r#"{
    key1: "value1"
    key2: "value2"
    key3: "value3"
    key4: "value4"
    key5: "value5x"
}"#;
    RULE.assert_replacement_contains(code, expected);
}

#[test]
fn fix_nested_record() {
    let code = r"let data = { id: 1, config: {nested: true} }";
    let expected = r#"{
    id: 1
    config: {nested: true}
}"#;
    RULE.assert_replacement_contains(code, expected);
}
