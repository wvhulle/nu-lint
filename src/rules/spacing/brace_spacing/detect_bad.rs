use super::RULE;

#[test]
fn test_space_before_closure_params() {
    let bad = "[[status]; [UP]] | all { |el| $el.status == UP }";
    RULE.assert_detects(bad);
}

#[test]
fn test_record_with_spaces() {
    // Records should not have spaces inside braces
    let bad_code = r#"let record = { name: "test" }"#;
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_record_with_inconsistent_spacing() {
    // Records should not have any spaces inside braces
    let bad_code = r#"let config = {host: "localhost" }"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_block_without_spaces() {
    // Blocks without parameters should have spaces
    let bad_code = "do {print 'test'}";
    RULE.assert_detects(bad_code);
}
