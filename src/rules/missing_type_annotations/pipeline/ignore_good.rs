use super::rule;

#[test]
fn test_typed_pipeline_input() {
    let good_code = r"
def double []: int -> int {
    $in * 2
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_typed_pipeline_output() {
    let good_code = r"
def create-list []: nothing -> list<int> {
    [1, 2, 3]
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_typed_both_input_output() {
    let good_code = r"
def transform []: list<int> -> list<int> {
    $in | each { |x| $x + 1 }
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_multiple_input_output_types() {
    let good_code = r"
def stringify []: [
    int -> string
    float -> string
] {
    $in | into string
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_no_pipeline_usage() {
    let good_code = r"
def add [a: int, b: int]: nothing -> int {
    $a + $b
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_typed_exported_command() {
    let good_code = r"
export def process []: string -> string {
    $in | str trim
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_typed_with_parameters() {
    let good_code = r"
def multiply [factor: int]: int -> int {
    $in * $factor
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_command_with_nothing_output() {
    let good_code = r"
def save-data [path: string]: any -> nothing {
    $in | save $path
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_string_to_record_signature() {
    let good_code = r"
def parse-json []: string -> record {
    $in | from json
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_table_processing() {
    let good_code = r"
def filter-rows []: table -> table {
    $in | where size > 100
}
";
    rule().assert_ignores(good_code);
}
