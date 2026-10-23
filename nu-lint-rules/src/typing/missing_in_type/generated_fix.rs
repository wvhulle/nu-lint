use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_fix_untyped_input_adds_any_for_output() {
    init_test_log();
    let bad_code = r"
def double [] {
    $in * 2
}
";
    RULE.assert_fixed_contains(bad_code, "[]: int -> any");
}

#[test]
fn test_fix_refines_any_input_type() {
    init_test_log();
    let bad_code = r"
def double []: any -> int {
    $in * 2
}
";
    RULE.assert_fixed_contains(bad_code, "[]: int -> int");
}

#[test]
fn test_fix_with_parameters() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    RULE.assert_fixed_contains(bad_code, "[factor: int]: int -> any");
}

#[test]
fn test_fix_preserves_existing_output_type() {
    let bad_code = r"
def transform []: any -> list<int> {
    $in | each { |x| $x + 1 }
}
";
    RULE.assert_fixed_contains(bad_code, "[]: list<any> -> list<int>");
}

#[test]
fn test_infers_list_input_from_each() {
    init_test_log();
    let bad_code = r"
def process_items [] {
    $in | each {|x| $x + 1}
}
";
    RULE.assert_fixed_contains(bad_code, "[]: list<any> -> any");
}

#[test]
fn test_infers_record_input_from_field_access() {
    let bad_code = r"
def get_name [] {
    $in.name
}
";
    RULE.assert_fixed_contains(bad_code, "[]: record -> any");
}

#[test]
fn test_fix_exported_function() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    RULE.assert_fixed_contains(bad_code, "[]: string -> any");
}
