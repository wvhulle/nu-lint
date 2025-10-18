use super::rule;

#[test]
fn test_not_is_empty_detected() {
    let bad_code = "if not ($list | is-empty) { echo 'has items' }";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_not_is_empty_in_if() {
    let bad_code = r#"
if not ($list | is-empty) {
    print "has items"
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_not_is_empty_in_assignment() {
    let bad_code = "let has_data = not ($data | is-empty)";

    rule().assert_detects(bad_code);
}
