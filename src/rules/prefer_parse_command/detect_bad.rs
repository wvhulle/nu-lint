use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_split_row_with_get_access() {
    let bad_code = r#"
let line = "Device AA:BB:CC:DD:EE:FF MyDevice"
let parts = ($line | split row " ")
let mac = ($parts | get 1)
let name = ($parts | skip 2 | str join " ")
"#;

    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_split_row_with_get_inline() {
    instrument();
    let bad_code = r#"
let ip = "192.168.1.100:8080" | split row ":" | get 0
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_with_skip() {
    instrument();
    let bad_code = r#"
let log = "[2024-01-15] INFO: Server started"
let parts = ($log | split row " ")
let message = ($parts | skip 2)
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_with_multiple_extractions() {
    instrument();
    let bad_code = r#"
let record = "Name: John, Age: 30, City: NYC"
let parts = ($record | split row ", ")
let name_part = ($parts | get 0)
let age_part = ($parts | get 1)
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_split_in_pipeline_with_get() {
    instrument();
    let bad_code = r#"
"user@example.com" | split row "@" | get 1
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_with_indexed_access() {
    instrument();
    let bad_code = r#"
let entry = "temperature:25.5:celsius"
let data = ($entry | split row ":")
let temp = ($data | get 1)
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_whitespace_split_with_index() {
    instrument();
    let bad_code = r#"
"foo   bar   baz" | split row " " | get 2
"#;
    rule().assert_detects(bad_code);
}
