use super::RULE;

#[test]
fn test_detect_split_space_first() {
    let bad_code = r#""hello world foo" | split row " " | first"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_space_last() {
    let bad_code = r#""hello world foo" | split row " " | last"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_space_first_with_1() {
    let bad_code = r#""hello world foo" | split row " " | first 1"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_space_last_with_1() {
    let bad_code = r#""hello world foo" | split row " " | last 1"#;
    RULE.assert_detects(bad_code);
}
