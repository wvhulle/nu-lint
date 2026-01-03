use super::RULE;

#[test]
fn test_fix_simple_split_get() {
    let bad_code = r#"
let split = ("a:b:c" | split row ":")
$split | get 0
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0}:{field1}:{field2}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field0");
}

#[test]
fn test_fix_split_get_different_index() {
    let bad_code = r#"
let parts = ("hello world" | split row " ")
$parts | get 1
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0} {field1}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field1");
}

#[test]
fn test_fix_with_filter() {
    let bad_code = r#"
let split = ("a,b,c" | split row "," | filter {|x| $x != "b"})
$split | get 0
"#;
    // Note: This fix won't preserve filter semantics perfectly, but suggests parse
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0},{field1},{field2}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field0");
}

#[test]
fn test_fix_with_skip() {
    let bad_code = r#"
let data = ("one:two:three" | split row ":")
$data | skip 1
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0}:{field1}:{field2}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field1");
}

#[test]
fn test_fix_multi_character_delimiter() {
    let bad_code = r#"
let parts = ("foo::bar::baz" | split row "::")
$parts | get 2
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0}::{field1}::{field2}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field2");
}

#[test]
fn test_fix_many_fields() {
    let bad_code = r#"
let items = ("one|two|three|four|five" | split row "|")
$items | get 3
"#;
    // | is a regex special char, so should use --regex
    RULE.assert_fixed_contains(bad_code, "parse --regex");
    RULE.assert_fixed_contains(bad_code, "get 0.field3");
}

#[test]
fn test_fix_datetime_pattern() {
    let bad_code = r#"
let parts = ("2025-11-20 08:21:51" | split row " ")
$parts | get 1
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0} {field1}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field1");
}

#[test]
fn test_fix_with_where_filter() {
    let bad_code = r#"
let split = ("a  b  c" | split row " " | where $it != "")
$split | get 1
"#;
    // The fix suggests parse but won't perfectly replicate the where filter
    // behavior
    RULE.assert_fixed_contains(
        bad_code,
        r#"parse "{field0} {field1} {field2} {field3} {field4}""#,
    );
    RULE.assert_fixed_contains(bad_code, "get 0.field1");
}

#[test]
fn test_fix_statements_in_between() {
    let bad_code = r#"
let split = ("a:b:c" | split row ":")
let x = 42
$split | get 2
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0}:{field1}:{field2}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field2");
}

#[test]
fn test_fix_regex_special_char_delimiter() {
    let bad_code = r#"
let parts = ("a.b.c.d" | split row ".")
$parts | get 0
"#;
    // Should use --regex with escaped delimiter
    RULE.assert_fixed_contains(bad_code, "parse --regex");
    RULE.assert_fixed_contains(bad_code, "get 0.field0");
}
