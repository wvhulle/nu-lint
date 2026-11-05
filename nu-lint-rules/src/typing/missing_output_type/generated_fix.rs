use nu_lint_analysis::log::init_test_log;

use super::RULE;

#[test]
fn test_fix_untyped_output_adds_any_for_input() {
    init_test_log();
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    RULE.assert_fixed_contains(bad_code, "[]: any -> list<int>");
}

#[test]
fn test_fix_refines_any_output_type() {
    init_test_log();
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
    init_test_log();
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

#[test]
fn fix_if_without_else_includes_nothing() {
    RULE.assert_fixed_contains(
        "def maybe-label [flag: bool] { if $flag { \"label\" } }",
        "oneof<string, nothing>",
    );
}

#[test]
fn fix_if_with_else_stays_single_output() {
    RULE.assert_fixed_contains(
        "def pick-label [flag: bool] { if $flag { \"yes\" } else { \"no\" } }",
        "-> string",
    );
}

#[test]
fn fix_does_not_guess_output_from_first_signature_of_unknown_input() {
    RULE.assert_fixed_not_contains("def pick [] { first }", "binary");
}

#[test]
fn fix_keeps_parameter_default_values() {
    RULE.assert_fixed_is(
        "def shift [offset = 1] { 2 }",
        "def shift [offset = 1]: any -> int { 2 }",
    );
}

#[test]
fn fix_keeps_closure_annotation_parseable() {
    RULE.assert_fixed_is(
        "def run-it [action: closure] { 1 }",
        "def run-it [action: closure]: any -> int { 1 }",
    );
}
