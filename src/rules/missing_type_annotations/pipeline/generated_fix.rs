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
    rule().assert_fix_contains(bad_code, "[]: int -> int");
    rule().assert_suggestion_contains(bad_code, "pipeline input and output type annotations");
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
    rule().assert_violation_count_exact(bad_code, 1);
    rule().assert_fix_contains(bad_code, "nothing -> table");
}

#[test]
fn test_fix_untyped_output() {
    instrument();
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> list<int>");
}

#[test]
fn test_fix_both_input_and_output() {
    instrument();
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
    rule().assert_fix_contains(bad_code, "[factor: int]: int -> int");
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
    rule().assert_fix_contains(bad_code, "data?");
    rule().assert_fix_contains(bad_code, "--verbose");
    rule().assert_fix_contains(bad_code, "string -> string");
}

#[test]
fn test_fix_exported_function() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    rule().assert_fix_contains(bad_code, "string -> string");
}

#[test]
fn test_infer_int_out() {
    instrument();
    let bad_code = r"
def get-value [] {
    42
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> int");
}

#[test]
fn test_infers_string_output() {
    let bad_code = r#"
def greet [] {
    "hello"
}
"#;
    rule().assert_fix_contains(bad_code, "[]: nothing -> string");
}

#[test]
fn test_infers_int_output() {
    let bad_code = r"
def get_count [] {
    42
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> int");
}

#[test]
fn test_infers_float_output() {
    let bad_code = r"
def get_pi [] {
    3.14
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> float");
}

#[test]
fn test_infers_bool_output() {
    instrument();
    let bad_code = r"
def is_ready [] {
    true
}
";
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
    rule().assert_fix_contains(bad_code, "[]: nothing -> record");
}

#[test]
fn test_infers_table_output() {
    let bad_code = r"
def get_data [] {
    [[name, age]; [Alice, 30], [Bob, 25]]
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> table");
}

#[test]
fn test_infers_output_from_to_json() {
    let bad_code = r"
def serialize [] {
    $in | to json
}
";
    rule().assert_fix_contains(bad_code, "[]: any -> string");
}

#[test]
fn test_infers_output_from_lines() {
    let bad_code = r"
def split_lines [] {
    $in | lines
}
";
    rule().assert_fix_contains(bad_code, "any -> list<string>");
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
    rule().assert_fix_contains(bad_code, "[]: any -> list<string>");
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
    instrument();
    let bad_code = r"
def get_path [] {
    /tmp/file.txt
}
";
    rule().assert_fix_contains(bad_code, "nothing -> path");
}

#[test]
fn test_longer_script() {
    instrument();
    let bad_code = r#"
def main [source_file: path] {
  let settings_file = $"($env.HOME)/.config/Code/User/settings.json"

  # Create directory if it doesn't exist
  mkdir ($settings_file | path dirname)

  # Determine if we should update the file
  let should_update = (
    not ($settings_file | path exists)
    or ($settings_file | path type) == "symlink"
    or (open --raw $source_file) != (open --raw $settings_file)
  )

  if $should_update {
    rm --force $settings_file
    cp $source_file $settings_file
    chmod 644 $settings_file
    print "Created/updated writable VSCode settings.json"
  }
}
"#;
    rule().assert_fix_contains(bad_code, "nothing -> any");
}

#[test]
fn test_if_with_side_effects_only() {
    instrument();
    let bad_code = r#"
def conditional_print [] {
    if true {
        print "yes"
    } else {
        print "no"
    }
}
"#;
    rule().assert_fix_contains(bad_code, "[]: nothing -> any");
}

#[test]
fn test_if_without_else_returns_nothing() {
    instrument();
    let bad_code = r#"
def conditional_action [flag: bool] {
    if $flag {
        print "flag is true"
    }
}
"#;
    rule().assert_fix_contains(bad_code, "[flag: bool]: nothing -> any");
}

#[test]
fn test_nested_if_with_side_effects() {
    instrument();
    let bad_code = r#"
def nested_conditional [] {
    if true {
        if false {
            print "inner"
        } else {
            mkdir /tmp/test
        }
    } else {
        rm /tmp/test
    }
}
"#;
    rule().assert_fix_contains(bad_code, "[]: nothing -> any");
}

#[test]
fn test_complex_body_with_let_and_side_effects() {
    instrument();
    let bad_code = r"
def complex_script [] {
    let x = 42
    print $x
    mkdir /tmp/dir
}
";
    rule().assert_fix_contains(bad_code, "[]: nothing -> nothing");
}
