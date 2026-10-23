use super::RULE;

#[test]
fn test_record_with_leading_space() {
    // Style guide incorrect example: "too many spaces before \"x\""
    let bad = "{ x: 1, y: 2}";
    RULE.assert_detects(bad);
}

#[test]
fn test_record_with_both_spaces() {
    let bad = "{ x: 1, y: 2 }";
    RULE.assert_detects(bad);
}

#[test]
fn test_record_with_trailing_space() {
    let bad = "{x: 1, y: 2 }";
    RULE.assert_detects(bad);
}

#[test]
fn test_record_assigned_with_spaces() {
    let bad = r#"let record = { name: "test" }"#;
    RULE.assert_count(bad, 1);
}

#[test]
fn test_record_with_variable_values_and_spaces() {
    let bad = "let config = { host: $host }";
    RULE.assert_detects(bad);
}

#[test]
fn test_inline_record_with_spaces() {
    let bad = "echo { key: value }";
    RULE.assert_detects(bad);
}
