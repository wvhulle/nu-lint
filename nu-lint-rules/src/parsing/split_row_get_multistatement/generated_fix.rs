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
fn test_no_fix_when_split_result_is_filtered() {
    RULE.assert_detects_without_fix(
        r#"
let split = ("a,b,c" | split row "," | filter {|x| $x != "b"})
$split | get 0
"#,
    );
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
fn test_no_fix_when_split_result_is_filtered_with_where() {
    RULE.assert_detects_without_fix(
        r#"
let split = ("a  b  c" | split row " " | where $it != "")
$split | get 1
"#,
    );
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

#[test]
fn test_fix_counts_escaped_delimiter_in_string_value() {
    let bad_code = r#"
let parts = ("a\tb\tc" | split row "\t")
$parts | get 2
"#;
    RULE.assert_fixed_contains(bad_code, "get 0.field2");
    RULE.assert_fixed_contains(bad_code, "field1");
}

#[test]
fn test_no_fix_when_input_is_not_a_literal() {
    RULE.assert_detects_without_fix(
        r#"
let line = "a:b:c"
let parts = ($line | split row ":")
$parts | get 2
"#,
    );
}

#[test]
fn test_fix_replaces_whole_declaration() {
    RULE.assert_fixed_is(
        "let parts = (\"a:b:c\" | split row \":\")\n$parts | get 1\n",
        "\"a:b:c\" | parse \"{field0}:{field1}:{field2}\" | get 0.field1\n",
    );
}

#[test]
fn test_no_fix_when_statement_sits_between() {
    RULE.assert_detects_without_fix(
        r#"
let parts = ("a:b:c" | split row ":")
print "keep me"
$parts | get 1
"#,
    );
}

#[test]
fn test_no_fix_when_variable_is_used_again() {
    RULE.assert_detects_without_fix(
        r#"
let parts = ("a:b:c" | split row ":")
$parts | get 0
print $parts
"#,
    );
}

#[test]
fn test_no_fix_when_input_has_earlier_stage() {
    RULE.assert_detects_without_fix(
        r#"
let parts = ("a:b:c" | str trim | split row ":")
$parts | get 1
"#,
    );
}
