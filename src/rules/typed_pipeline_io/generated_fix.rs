use crate::log::instrument;

use super::rule;

#[test]
fn test_fix_untyped_input() {
    instrument();
    let bad_code = r"
def double [] {
    $in * 2
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: any -> any");
    rule().assert_suggestion_contains(bad_code, "pipeline input type annotation");
}

#[test]
fn test_fix_untyped_output() {
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> list<int>");
}

#[test]
fn test_fix_both_input_and_output() {
    let bad_code = r"
def transform [] {
    $in | each { |x| $x + 1 }
}
";
    rule().assert_fix_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_fix_with_parameters() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[factor: int]: any -> any");
}

#[test]
fn test_fix_description_mentions_type_annotations() {
    let bad_code = "def double [] { $in * 2 }";
    rule().assert_fix_description_contains(bad_code, "type annotations");
}

#[test]
fn test_fix_preserves_optional_parameters() {
    let bad_code = r"
def process [data?, --verbose] {
    $in | str trim
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "data?");
    rule().assert_fix_contains(bad_code, "--verbose");
    rule().assert_fix_contains(bad_code, ": any -> any");
}

#[test]
fn test_fix_exported_function() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: any -> any");
}

#[test]
fn test_infer_int_out() {
    instrument();
    let bad_code = r"
def get-value [] {
    42
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> int");
}

#[test]
fn test_infers_string_output() {
    let bad_code = r#"
def greet [] {
    "hello"
}
"#;
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> string");
}

#[test]
fn test_infers_int_output() {
    let bad_code = r"
def get_count [] {
    42
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> int");
}

#[test]
fn test_infers_float_output() {
    let bad_code = r"
def get_pi [] {
    3.14
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> float");
}

#[test]
fn test_infers_bool_output() {
    let bad_code = r"
def is_ready [] {
    true
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> bool");
}

#[test]
fn test_infers_list_output() {
    instrument();
    let bad_code = r"
def get_items [] {
    [1, 2, 3]
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> list");
}

#[test]
fn test_infers_record_output() {
    let bad_code = r#"
def get_config [] {
    {name: "test", value: 42}
}
"#;
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> record");
}

#[test]
fn test_infers_table_output() {
    let bad_code = r"
def get_data [] {
    [[name, age]; [Alice, 30], [Bob, 25]]
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> table");
}

#[test]
fn test_infers_output_from_to_json() {
    let bad_code = r"
def serialize [] {
    $in | to json
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: any -> string");
}

#[test]
fn test_infers_output_from_lines() {
    let bad_code = r"
def split_lines [] {
    $in | lines
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "string -> list<string>");
}

#[test]
fn test_infers_output_from_where() {
    let bad_code = r"
def filter_items [] {
    $in | where {|x| $x > 5}
}
";
    rule().assert_fix_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_infers_output_from_length() {
    let bad_code = r"
def count_items [] {
    $in | length
}
";
    rule().assert_fix_contains(bad_code, "list<any> -> int");
}

#[test]
fn test_infers_output_from_is_empty() {
    let bad_code = r"
def check_empty [] {
    $in | is-empty
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: any -> bool");
}

#[test]
fn test_infers_record_input_from_field_access() {
    let bad_code = r"
def get_name [] {
    $in.name
}
";
    rule().assert_fix_contains(bad_code, "[]: record -> any");
}

#[test]
fn test_infers_list_input_from_each() {
    instrument();
    let bad_code = r"
def process_items [] {
    $in | each {|x| $x + 1}
}
";
    rule().assert_fix_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_infers_string_input_from_lines() {
    let bad_code = r"
def split_text [] {
    $in | lines
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: string -> list");
}

#[test]
fn test_infers_both_input_and_output_types() {
    let bad_code = r"
def process [] {
    $in | each {|x| $x * 2}
}
";
    rule().assert_fix_contains(bad_code, "[]: list<any> -> list<any>");
}

#[test]
fn test_infers_with_parameters_and_types() {
    let bad_code = r"
def multiply [factor: int] {
    $in | each {|x| $x * $factor}
}
";
    rule().assert_fix_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_fallback_to_any_for_complex_output() {
    let bad_code = r#"
def complex [] {
    if true { "string" } else { 42 }
}
"#;
    rule().assert_fix_contains(bad_code, "[]: nothing -> any");
}

#[test]
fn test_path_output_from_filepath() {
    let bad_code = r"
def get_path [] {
    /tmp/file.txt
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> path");
}
