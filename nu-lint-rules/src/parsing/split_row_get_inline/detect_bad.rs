use super::RULE;

#[test]
fn test_detect_split_row_with_get_inline() {
    let bad_code = r#"
let ip = "192.168.1.100:8080" | split row ":" | get 0
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_in_pipeline_with_get() {
    let bad_code = r#"
"user@example.com" | split row "@" | get 1
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_whitespace_split_with_index() {
    let bad_code = r#"
"foo   bar   baz" | split row " " | get 2
"#;
    RULE.assert_detects(bad_code);
}
