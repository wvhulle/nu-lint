use super::RULE;

#[test]
fn test_detect_simple_split_get_across_statements() {
    crate::log::init_log();
    let bad_code = r#"
let split = ("a:b:c" | split row ":")
$split | get 0
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_with_space_delimiter() {
    let bad_code = r#"
let parts = ("hello world" | split row " ")
$parts | get 1
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_with_where_filter() {
    let bad_code = r#"
let split = ("a  b  c" | split row " " | where $it != "")
$split | get 1
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_with_filter_command() {
    let bad_code = r#"
let split = ("a,b,c" | split row "," | filter {|x| $x != "b"})
$split | get 0
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_statements_between() {
    let bad_code = r#"
let split = ("a:b:c" | split row ":")
let x = 42
$split | get 2
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_with_skip_instead_of_get() {
    let bad_code = r#"
let split = ("a:b:c" | split row ":")
$split | skip 1
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_user_reported_case() {
    let bad_code = r#"
let rest = "2025-11-20 08:21:51 +01:00"
let split = ($rest | split row " " | where $it != "")
$split | get 1
"#;
    RULE.assert_detects(bad_code);
}
