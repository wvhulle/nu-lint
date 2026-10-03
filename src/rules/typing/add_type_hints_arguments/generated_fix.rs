use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_infer_string_type_from_string_operations() {
    init_test_log();
    let bad_code = r"
def process [text] {
    $text | str trim
}
";
    RULE.assert_fixed_contains(bad_code, "text: string");
}

#[test]
fn test_infer_int_type_from_math_operations() {
    let bad_code = r"
def add_ten [num] {
    $num + 10
}
";
    RULE.assert_fixed_contains(bad_code, "num: int");
}

#[test]
fn test_infer_list_type_from_each() {
    let bad_code = r"
def process [items] {
    $items | each { |x| $x + 1 }
}
";
    RULE.assert_fixed_contains(bad_code, "items: list");
}

#[test]
fn test_infer_record_type_from_field_access() {
    let bad_code = r"
def get_name [person] {
    $person.name
}
";
    RULE.assert_fixed_contains(bad_code, "person: record");
}

#[test]
fn test_fallback_to_any_when_unknown() {
    let bad_code = r"
def identity [value] {
    $value
}
";
    RULE.assert_fixed_contains(bad_code, "value: any");
}

#[test]
fn test_multiple_params_with_inference() {
    let bad_code = r"
def combine [text, num] {
    $text | str trim
    $num + 5
}
";
    RULE.assert_fixed_contains(bad_code, "text: string");
    RULE.assert_fixed_contains(bad_code, "num: int");
}

#[test]
fn test_preserves_existing_types() {
    let bad_code = r"
def process [text: string, num] {
    $num + 1
}
";
    RULE.assert_fixed_contains(bad_code, "num: int");
    RULE.assert_fixed_contains(bad_code, "text: string");
}

#[test]
fn test_optional_parameter_with_inference() {
    init_test_log();
    let bad_code = r"
def greet [name?] {
    $name | str trim
}
";
    RULE.assert_fixed_contains(bad_code, "name?: string");
}

#[test]
fn test_rest_parameter_with_inference() {
    let bad_code = r"
def sum [...nums] {
    $nums | each { |n| $n + 1 }
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def sum [...nums: any] {
    $nums | each { |n| $n + 1 }
}
",
    );
}

#[test]
fn test_fix_preserves_flags() {
    let bad_code = r"
def f [x, --verbose] {
    $x + 1
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [x: int, --verbose] {
    $x + 1
}
",
    );
}

#[test]
fn test_complex_body_with_if_statement() {
    init_test_log();
    let bad_code = r"
def process [value] {
    if ($value > 10) {
        $value + 5
    } else {
        $value - 5
    }
}
";
    RULE.assert_fixed_contains(bad_code, "value: int");
}

#[test]
fn test_complex_body_with_nested_call() {
    let bad_code = r"
def transform [data] {
    $data | str trim | str upcase
}
";
    RULE.assert_fixed_contains(bad_code, "data: string");
}

#[test]
fn test_parameter_used_in_subexpression() {
    init_test_log();
    let bad_code = r"
def calculate [x] {
    let result = ($x + 10)
    $result
}
";
    RULE.assert_fixed_contains(bad_code, "x: int");
}

#[test]
fn test_parameter_with_closure() {
    let bad_code = r"
def apply [items] {
    $items | where {|x| $x > 5}
}
";
    RULE.assert_fixed_contains(bad_code, "items: list");
}

#[test]
fn test_parameter_with_field_access_in_closure() {
    let bad_code = r"
def get_names [people] {
    $people | each {|p| $p.name}
}
";
    RULE.assert_fixed_contains(bad_code, "people: list");
}

#[test]
fn test_nested_function_with_inference() {
    init_test_log();
    let bad_code = r#"
def outer [] {
    def inner [param] {
        $param | str trim
    }
    inner "test"
}
"#;
    RULE.assert_fixed_contains(bad_code, "param: string");
}

#[test]
fn test_multiple_params_complex_usage() {
    let bad_code = r"
def process [text, items, count] {
    $text | str trim
    $items | each { |x| $x + 1 }
    $count + 10
}
";
    RULE.assert_fixed_contains(bad_code, "text: string");
    RULE.assert_fixed_contains(bad_code, "items: list");
    RULE.assert_fixed_contains(bad_code, "count: int");
}

#[test]
fn test_param_used_in_comparison() {
    let bad_code = r"
def is_greater [value] {
    $value > 100
}
";
    RULE.assert_fixed_contains(bad_code, "value: int");
}

#[test]
fn test_param_in_binary_operation() {
    let bad_code = r"
def multiply [a, b] {
    $a * $b
}
";
    RULE.assert_fixed_contains(bad_code, "a: int");
    RULE.assert_fixed_contains(bad_code, "b: int");
}

#[test]
fn test_mixed_typed_and_untyped_params() {
    let bad_code = r"
def process [data: string, count, items] {
    $data | str trim
    $count + 1
    $items | each { |x| $x }
}
";
    RULE.assert_fixed_contains(bad_code, "data: string");
    RULE.assert_fixed_contains(bad_code, "count: int");
    RULE.assert_fixed_contains(bad_code, "items: list");
}

#[test]
fn test_parameter_with_str_replace() {
    let bad_code = r#"
def clean [text] {
    $text | str replace "old" "new"
}
"#;
    RULE.assert_fixed_contains(bad_code, "text: string");
}

#[test]
fn test_parameter_with_str_downcase() {
    let bad_code = r"
def lowercase [input] {
    $input | str downcase
}
";
    RULE.assert_fixed_contains(bad_code, "input: string");
}

#[test]
fn test_parameter_with_str_upcase() {
    let bad_code = r"
def uppercase [input] {
    $input | str upcase
}
";
    RULE.assert_fixed_contains(bad_code, "input: string");
}

#[test]
fn test_parameter_with_filter() {
    let bad_code = r"
def filter_items [data] {
    $data | filter {|x| $x > 5}
}
";
    RULE.assert_fixed_contains(bad_code, "data: list");
}

#[test]
fn test_parameter_with_reduce() {
    let bad_code = r"
def sum_all [numbers] {
    $numbers | reduce {|it, acc| $acc + $it}
}
";
    RULE.assert_fixed_contains(bad_code, "numbers: list");
}

#[test]
fn test_parameter_with_append() {
    let bad_code = r"
def add_item [items] {
    $items | append 42
}
";
    // append accepts 'any' as input, so inference will yield 'any'
    RULE.assert_fixed_contains(bad_code, "items: any");
}

#[test]
fn test_parameter_with_prepend() {
    let bad_code = r"
def add_first [collection] {
    $collection | prepend 0
}
";
    // prepend accepts 'any' as input, so inference will yield 'any'
    RULE.assert_fixed_contains(bad_code, "collection: any");
}

#[test]
fn test_complex_body_with_let_statements() {
    init_test_log();
    let bad_code = r"
def compute [x] {
    let doubled = ($x * 2)
    let tripled = ($x * 3)
    $doubled + $tripled
}
";
    RULE.assert_fixed_contains(bad_code, "x: int");
}

#[test]
fn test_complex_body_with_multiple_pipelines() {
    init_test_log();
    let bad_code = r"
def process_data [data] {
    let cleaned = ($data | str trim)
    let upper = ($cleaned | str upcase)
    $upper
}
";
    RULE.assert_fixed_contains(bad_code, "data: string");
}

#[test]
fn test_exported_function_with_inference() {
    let bad_code = r"
export def process [text] {
    $text | str trim
}
";
    RULE.assert_fixed_contains(bad_code, "text: string");
}

#[test]
fn test_parameter_in_nested_block() {
    init_test_log();
    let bad_code = r"
def outer [value] {
    do {
        $value + 10
    }
}
";
    RULE.assert_fixed_contains(bad_code, "value: int");
}

#[test]
fn test_all_param_types_together() {
    init_test_log();
    let bad_code = r"
def complex [required, optional?, ...rest] {
    $required | str trim
    $optional | each { |x| $x }
    $rest | where {|x| $x > 0}
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def complex [required: string, optional?: list<any>, ...rest: any] {
    $required | str trim
    $optional | each { |x| $x }
    $rest | where {|x| $x > 0}
}
",
    );
}

#[test]
fn test_deeply_nested_if_statements() {
    init_test_log();
    let bad_code = r"
def nested_logic [val] {
    if ($val > 0) {
        if ($val > 10) {
            $val * 2
        } else {
            $val + 1
        }
    } else {
        $val - 1
    }
}
";
    RULE.assert_fixed_contains(bad_code, "val: int");
}

#[test]
fn test_parameter_with_str_contains() {
    let bad_code = r#"
def has_word [text] {
    $text | str contains "word"
}
"#;
    RULE.assert_fixed_contains(bad_code, "text: string");
}

#[test]
fn test_explicit_any_is_replaced_by_inferred_type() {
    let bad_code = r"
def f [x: any, y] {
    $x + 1
    $y + 1
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [x: int, y: int] {
    $x + 1
    $y + 1
}
",
    );
}

#[test]
fn test_explicit_any_on_optional_param_is_replaced() {
    let bad_code = r"
def f [x?: any] {
    $x | str trim
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [x?: string] {
    $x | str trim
}
",
    );
}

#[test]
fn test_explicit_any_without_better_inference_has_no_fix() {
    let bad_code = r"
def f [x: any] {
    $x | describe
}
";
    RULE.assert_detects_without_fix(bad_code);
}

#[test]
fn test_fix_keeps_input_output_types() {
    let bad_code = r"
def f [x]: nothing -> int {
    $x + 1
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [x: int]: nothing -> int {
    $x + 1
}
",
    );
}

#[test]
fn test_fix_keeps_multiline_signature_with_comments() {
    let bad_code = r"
def f [
    x # the input
    --verbose # be loud
] {
    $x + 1
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [
    x: int # the input
    --verbose # be loud
] {
    $x + 1
}
",
    );
}

#[test]
fn test_param_passed_to_print_is_not_nothing() {
    let bad_code = r"
def f [x] {
    print $x
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [x: any] {
    print $x
}
",
    );
}

#[test]
fn test_param_type_from_called_command_parameter() {
    let bad_code = r"
def f [path] {
    mkdir $path
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [path: oneof<glob, string>] {
    mkdir $path
}
",
    );
}

#[test]
fn test_param_type_from_positional_after_named_flag() {
    let bad_code = r"
def f [replacement] {
    str replace --all 'a' $replacement 'abc'
}
";
    RULE.assert_fixed_is(
        bad_code,
        r"
def f [replacement: oneof<string, closure>] {
    str replace --all 'a' $replacement 'abc'
}
",
    );
}
