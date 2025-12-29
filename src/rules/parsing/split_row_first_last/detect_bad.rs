use super::RULE;

#[test]
fn test_detect_split_row_first() {
    let bad_code = r#""a:b:c" | split row ":" | first"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_last() {
    let bad_code = r#""a:b:c" | split row ":" | last"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_first_with_1() {
    let bad_code = r#""path/to/file" | split row "/" | first 1"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_last_with_1() {
    let bad_code = r#""path/to/file.txt" | split row "/" | last 1"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_space_first() {
    let bad_code = r#""hello world foo" | split row " " | first"#;
    RULE.assert_detects(bad_code);
}
