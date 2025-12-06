use super::rule;
use crate::log::instrument;

#[test]
fn test_fix_untyped_input() {
    instrument();
    let bad_code = r"
def double [] {
    $in * 2
}
";
    rule().assert_replacement_contains(bad_code, "[]: int -> int");
    rule().assert_help_contains(bad_code, "pipeline input and output type annotations");
}

#[test]
fn test_missing_pipeline_annot_git() {
    instrument();

    let bad_code = r#"
export def "git age" [] {
  git branch | lines | str substring 2.. | wrap name | insert last_commit {
    get name | each {
      git show $in --no-patch --format=%as | into datetime
    }
  } | sort-by last_commit
}
"#;
    rule().assert_count(bad_code, 1);
    rule().assert_replacement_contains(bad_code, "nothing -> table");
}

#[test]
fn test_fix_untyped_output() {
    instrument();
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    rule().assert_replacement_contains(bad_code, "[]: nothing -> list<int>");
}

#[test]
fn test_infer_float() {
    instrument();
    let bad_code = r#"
def time_to_hours [time_str: string] {
    let parts = ($time_str | split row ":")
    let hour = ($parts.0 | into float)
    let minute = ($parts.1 | into float)
    # Ignore seconds for simplicity
    $hour + ($minute / 60.0)
}"#;
    rule().assert_replacement_contains(bad_code, "nothing -> float");
}

#[test]
fn test_fix_both_input_and_output() {
    instrument();
    let bad_code = r"
def transform [] {
    $in | each { |x| $x + 1 }
}
";
    rule().assert_replacement_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_fix_with_parameters() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    rule().assert_replacement_contains(bad_code, "[factor: int]: int -> int");
}

#[test]
fn test_fix_description_mentions_type_annotations() {
    let bad_code = "def double [] { $in * 2 }";
    rule().assert_fix_explanation_contains(bad_code, "type annotations");
}

#[test]
fn test_fix_preserves_optional_parameters() {
    let bad_code = r"
def process [data?, --verbose] {
    $in | str trim
}
";
    rule().assert_replacement_contains(bad_code, "data?");
    rule().assert_replacement_contains(bad_code, "--verbose");
    rule().assert_replacement_contains(bad_code, "string -> string");
}

#[test]
fn test_fix_exported_function() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    rule().assert_replacement_contains(bad_code, "string -> string");
}

#[test]
fn test_infer_int_out() {
    instrument();
    let bad_code = r"
def get-value [] {
    42
}
";
    rule().assert_replacement_contains(bad_code, "[]: nothing -> int");
}

#[test]
fn test_infers_string_output() {
    let bad_code = r#"
def greet [] {
    "hello"
}
"#;
    rule().assert_replacement_contains(bad_code, "[]: nothing -> string");
}

#[test]
fn test_infers_int_output() {
    let bad_code = r"
def get_count [] {
    42
}
";
    rule().assert_replacement_contains(bad_code, "[]: nothing -> int");
}

#[test]
fn test_infers_float_output() {
    let bad_code = r"
def get_pi [] {
    3.14
}
";
    rule().assert_replacement_contains(bad_code, "[]: nothing -> float");
}

#[test]
fn test_infers_bool_output() {
    instrument();
    let bad_code = r"
def is_ready [] {
    true
}
";
    rule().assert_replacement_contains(bad_code, "[]: nothing -> bool");
}

#[test]
fn test_infers_list_output() {
    instrument();
    let bad_code = r"
def get_items [] {
    [1, 2, 3]
}
";
    rule().assert_replacement_contains(bad_code, "[]: nothing -> list");
}

#[test]
fn test_infers_record_output() {
    let bad_code = r#"
def get_config [] {
    {name: "test", value: 42}
}
"#;
    rule().assert_replacement_contains(bad_code, "[]: nothing -> record");
}

#[test]
fn test_infers_table_output() {
    let bad_code = r"
def get_data [] {
    [[name, age]; [Alice, 30], [Bob, 25]]
}
";
    rule().assert_replacement_contains(bad_code, "[]: nothing -> table");
}

#[test]
fn test_infers_output_from_to_json() {
    let bad_code = r"
def serialize [] {
    $in | to json
}
";
    rule().assert_replacement_contains(bad_code, "[]: any -> string");
}

#[test]
fn test_infers_output_from_lines() {
    let bad_code = r"
def split_lines [] {
    $in | lines
}
";
    rule().assert_replacement_contains(bad_code, "any -> list<string>");
}

#[test]
fn test_infers_output_from_where() {
    let bad_code = r"
def filter_items [] {
    $in | where {|x| $x > 5}
}
";
    rule().assert_replacement_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_infers_output_from_length() {
    let bad_code = r"
def count_items [] {
    $in | length
}
";
    rule().assert_replacement_contains(bad_code, "list<any> -> int");
}

#[test]
fn test_infers_output_from_is_empty() {
    let bad_code = r"
def check_empty [] {
    $in | is-empty
}
";
    rule().assert_replacement_contains(bad_code, "[]: any -> bool");
}

#[test]
fn test_infers_record_input_from_field_access() {
    let bad_code = r"
def get_name [] {
    $in.name
}
";
    rule().assert_replacement_contains(bad_code, "[]: record -> any");
}

#[test]
fn test_infers_list_input_from_each() {
    instrument();
    let bad_code = r"
def process_items [] {
    $in | each {|x| $x + 1}
}
";
    rule().assert_replacement_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_infers_string_input_from_lines() {
    let bad_code = r"
def split_text [] {
    $in | lines
}
";
    rule().assert_replacement_contains(bad_code, "[]: any -> list<string>");
}

#[test]
fn test_infers_both_input_and_output_types() {
    let bad_code = r"
def process [] {
    $in | each {|x| $x * 2}
}
";
    rule().assert_replacement_contains(bad_code, "[]: list<any> -> list<any>");
}

#[test]
fn test_infers_with_parameters_and_types() {
    let bad_code = r"
def multiply [factor: int] {
    $in | each {|x| $x * $factor}
}
";
    rule().assert_replacement_contains(bad_code, "list<any> -> list<any>");
}

#[test]
fn test_fallback_to_any_for_complex_output() {
    instrument();
    let bad_code = r#"
def complex [] {
    if true { "string" } else { 42 }
}
"#;
    rule().assert_replacement_contains(bad_code, "[]: nothing -> string");
}

#[test]
fn test_preserves_multiline_function_signature() {
    instrument();
    let bad_code = r"
def calculate-brightness [
  current: float
  times: record
  --min: float
  --max: float
  --offset: int
] {
  let offset_hours = $offset / 60.0
  let dawn = ($times.dawn | time-to-hours) + $offset_hours
  $dawn
}
";
    // Should preserve the multiline formatting with newlines and indentation
    rule().assert_replacement_contains(bad_code, "\n  current: float\n");
    rule().assert_replacement_contains(bad_code, "\n  times: record\n");
    rule().assert_replacement_contains(bad_code, "\n  --min: float\n");
    rule().assert_replacement_contains(bad_code, "\n  --max: float\n");
    rule().assert_replacement_contains(bad_code, "\n  --offset: int\n");
}

#[test]
fn test_preserves_multiline_with_optional_params() {
    instrument();
    let bad_code = r"
export def process-data [
  input: string
  output: string
  --verbose: bool
  --format: string
] {
  $input | parse
}
";
    // Should preserve the multiline formatting
    rule().assert_replacement_contains(bad_code, "\n  input: string\n");
    rule().assert_replacement_contains(bad_code, "\n  output: string\n");
    rule().assert_replacement_contains(bad_code, "\n  --verbose: bool\n");
    rule().assert_replacement_contains(bad_code, "\n  --format: string\n");
    rule().assert_replacement_contains(bad_code, "nothing -> table");
}

#[test]
fn test_single_line_signature_stays_single_line() {
    instrument();
    let bad_code = r"
def transform [data: string, options: record] {
    $data | str trim
}
";
    // Should keep single-line format
    rule().assert_replacement_contains(bad_code, "[data: string, options: record]:");
    rule().assert_replacement_contains(bad_code, "nothing -> string");
}
