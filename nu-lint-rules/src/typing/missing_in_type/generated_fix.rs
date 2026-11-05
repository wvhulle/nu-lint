use nu_lint_analysis::log::init_test_log;

use super::RULE;

#[test]
fn test_fix_untyped_input_adds_any_for_output() {
    init_test_log();
    let bad_code = r"
def double [] {
    $in * 2
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def double []: any -> any {
    $in * 2
}
",
    );
}

#[test]
fn test_fix_refines_any_input_type() {
    init_test_log();
    let bad_code = r#"
def is_mac []: any -> bool {
    $in =~ "mac"
}
"#;
    RULE.assert_fixed_is(
        bad_code,
        r#"
def is_mac []: string -> bool {
    $in =~ "mac"
}
"#,
    );
}

#[test]
fn test_fix_with_parameters() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def multiply [factor: int]: any -> any {
    $in * $factor
}
",
    );
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

#[test]
fn fix_keeps_parameter_default_values() {
    RULE.assert_fixed_is(
        "def shift [offset = 1] { $in | length }",
        "def shift [offset = 1]: list<any> -> any { $in | length }",
    );
}

#[test]
fn no_fix_when_annotated_any_cannot_be_refined() {
    RULE.assert_detects_without_fix("def double []: any -> int { $in * 2 }");
}
