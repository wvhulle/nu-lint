use crate::log::instrument;

use super::rule;

#[test]
fn test_detect_manual_string_splitting_device() {
    let bad_code = r#"
let line = "Device AA:BB:CC:DD:EE:FF MyDevice"
let parts = ($line | split row " ")
let mac = ($parts | get 1)
let name = ($parts | skip 2 | str join " ")
"#;

    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_manual_string_splitting_user_data() {
    instrument();
    let bad_code = r#"
let data = "user:john:1000"
let fields = ($data | split row ":")
let username = ($fields | get 0)
"#;

    rule().assert_detects(bad_code);
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
fn test_detect_split_row_multiple_gets() {
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
fn test_detect_nested_split_get() {
    instrument();
    let bad_code = r#"
let config = "key=value"
let pair = ($config | split row "=")
let key = ($pair | get 0)
let val = ($pair | get 1)
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
fn test_detect_split_row_with_first() {
    instrument();
    let bad_code = r#"
let entry = "temperature:25.5:celsius"
let data = ($entry | split row ":")
let temp = ($data | get 1)
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_log_parsing_manual() {
    instrument();
    let bad_code = r#"
let log_line = "ERROR [module:function] Something went wrong"
let parts = ($log_line | split row " ")
let level = ($parts | get 0)
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
