use super::rule;

#[test]
fn ignore_properly_typed_input() {
    let good_code = r"
def double []: int -> int {
    $in * 2
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_properly_typed_output() {
    let good_code = r"
def create-list []: nothing -> list<int> {
    [1, 2, 3]
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_fully_typed_pipeline() {
    let good_code = r"
def transform []: list<int> -> list<int> {
    $in | each { |x| $x + 1 }
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_union_type_signatures() {
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
fn ignore_function_without_pipeline_usage() {
    let good_code = r"
def add [a: int, b: int]: nothing -> int {
    $a + $b
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_exported_function_with_types() {
    let good_code = r"
export def process []: string -> string {
    $in | str trim
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_typed_pipeline_with_params() {
    let good_code = r"
def multiply [factor: int]: int -> int {
    $in * $factor
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_function_returning_nothing() {
    let good_code = r"
def save-data [path: string]: any -> nothing {
    $in | save $path
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_string_to_record_conversion() {
    let good_code = r"
def parse-json []: string -> record {
    $in | from json
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_table_to_table_processing() {
    let good_code = r"
def filter-rows []: table -> table {
    $in | where size > 100
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_print_only_command() {
    let good_code = r#"
def main [] {
    print "Solar-based brightness manager"
    print "Use --help to see available subcommands"
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_side_effect_only_command() {
    let good_code = r#"
def setup [] {
    mkdir ~/.config/myapp
    print "Setup complete"
}
"#;
    rule().assert_ignores(good_code);
}
