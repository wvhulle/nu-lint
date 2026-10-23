use super::RULE;

#[test]
fn fix_long_single_line_record() {
    let code = r#"let config = {name: "very long name here", explanation: "very long description text", version: "1.0.0"}"#;
    let expected = r#"{
    name: "very long name here"
    explanation: "very long description text"
    version: "1.0.0"
}"#;
    RULE.assert_fixed_contains(code, expected);
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
    RULE.assert_fixed_contains(code, expected);
}

#[test]
fn fix_deeply_nested_record() {
    let code = r"let data = {a: 1, b: {c: 2, d: {e: 3}}}";
    let expected = r#"{
    a: 1
    b: {c: 2, d: {e: 3}}
}"#;
    RULE.assert_fixed_contains(code, expected);
}

#[test]
fn fix_long_nested_record() {
    let code = r#"let data = {name: "long name here", config: {option1: true, option2: false}}"#;
    let expected = r#"{
    name: "long name here"
    config: {option1: true, option2: false}
}"#;
    RULE.assert_fixed_contains(code, expected);
}
