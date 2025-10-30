use super::rule;
#[test]
fn test_space_before_closure_params() {
    let bad = "[[status]; [UP]] | all { |el| $el.status == UP }";

    rule().assert_detects(bad);
}

#[test]
fn test_inconsistent_spacing_space_after_only() {
    let bad_code = r#"let record = { name: "test"}"#;

    rule().assert_violation_count(bad_code, 1);
}

#[test]
fn test_inconsistent_spacing_space_before_only() {
    let bad_code = r#"let config = {host: "localhost" }"#;

    rule().assert_detects(bad_code);
}
