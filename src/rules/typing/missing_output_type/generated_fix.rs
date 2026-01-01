use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_fix_untyped_output_adds_any_for_input() {
    init_env_log();
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    RULE.assert_fixed_contains(bad_code, "[]: any -> list<int>");
}

#[test]
fn test_fix_refines_any_output_type() {
    init_env_log();
    let bad_code = r"
def create-list []: nothing -> any {
    [1, 2, 3]
}
";
    RULE.assert_fixed_contains(bad_code, "[]: nothing -> list<int>");
}

#[test]
fn test_fix_preserves_existing_input_type() {
    let bad_code = r"
def transform []: list<any> -> any {
    $in | each { |x| $x + 1 }
}
";
    RULE.assert_fixed_contains(bad_code, "[]: list<any> -> list<any>");
}

#[test]
fn test_infers_string_output() {
    let bad_code = r#"
def greet [] {
    "hello"
}
"#;
    RULE.assert_fixed_contains(bad_code, "[]: any -> string");
}

#[test]
fn test_infers_int_output() {
    let bad_code = r"
def get_count [] {
    42
}
";
    RULE.assert_fixed_contains(bad_code, "[]: any -> int");
}

#[test]
fn test_infers_float_output() {
    let bad_code = r"
def get_pi [] {
    3.14
}
";
    RULE.assert_fixed_contains(bad_code, "[]: any -> float");
}

#[test]
fn test_infers_bool_output() {
    init_env_log();
    let bad_code = r"
def is_ready [] {
    true
}
";
    RULE.assert_fixed_contains(bad_code, "[]: any -> bool");
}

#[test]
fn test_infers_record_output() {
    let bad_code = r#"
def get_config [] {
    {name: "test", value: 42}
}
"#;
    RULE.assert_fixed_contains(bad_code, "[]: any -> record");
}

#[test]
fn test_infers_table_output() {
    let bad_code = r"
def get_data [] {
    [[name, age]; [Alice, 30], [Bob, 25]]
}
";
    RULE.assert_fixed_contains(bad_code, "[]: any -> table");
}

#[test]
fn test_infers_output_from_to_json() {
    let bad_code = r"
def serialize [] {
    $in | to json
}
";
    RULE.assert_fixed_contains(bad_code, "[]: any -> string");
}

#[test]
fn test_infers_output_from_lines() {
    let bad_code = r"
def split_lines [] {
    $in | lines
}
";
    RULE.assert_fixed_contains(bad_code, "any -> list<string>");
}

#[test]
fn test_fix_with_parameters() {
    let bad_code = r#"
def time_to_hours [time_str: string] {
    let parts = ($time_str | split row ":")
    let hour = ($parts.0 | into float)
    let minute = ($parts.1 | into float)
    $hour + ($minute / 60.0)
}"#;
    RULE.assert_fixed_contains(bad_code, "[time_str: string]: any -> float");
}
