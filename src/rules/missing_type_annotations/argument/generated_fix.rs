use super::rule;
use crate::log::instrument;

#[test]
fn test_infer_string_type_from_string_operations() {
    let bad_code = r"
def process [text] {
    $text | str trim
}
";
    rule().assert_fix_contains(bad_code, "text: string");
}

#[test]
fn test_infer_int_type_from_math_operations() {
    let bad_code = r"
def add_ten [num] {
    $num + 10
}
";
    rule().assert_fix_contains(bad_code, "num: int");
}

#[test]
fn test_infer_list_type_from_each() {
    let bad_code = r"
def process [items] {
    $items | each { |x| $x + 1 }
}
";
    rule().assert_fix_contains(bad_code, "items: list");
}

#[test]
fn test_infer_record_type_from_field_access() {
    let bad_code = r"
def get_name [person] {
    $person.name
}
";
    rule().assert_fix_contains(bad_code, "person: record");
}

#[test]
fn test_fallback_to_any_when_unknown() {
    let bad_code = r"
def identity [value] {
    $value
}
";
    rule().assert_fix_contains(bad_code, "value: any");
}

#[test]
fn test_multiple_params_with_inference() {
    let bad_code = r"
def combine [text, num] {
    $text | str trim
    $num + 5
}
";
    rule().assert_fix_contains(bad_code, "text: string");
    rule().assert_fix_contains(bad_code, "num: int");
}

#[test]
fn test_preserves_existing_types() {
    let bad_code = r"
def process [text: string, num] {
    $num + 1
}
";
    rule().assert_fix_contains(bad_code, "num: int");
    rule().assert_fix_contains(bad_code, "text: string");
}

#[test]
fn test_optional_parameter_with_inference() {
    instrument();
    let bad_code = r"
def greet [name?] {
    $name | str trim
}
";
    rule().assert_fix_contains(bad_code, "name?: string");
}

#[test]
fn test_rest_parameter_with_inference() {
    let bad_code = r"
def sum [...nums] {
    $nums | each { |n| $n + 1 }
}
";
    rule().assert_fix_contains(bad_code, "...nums: list");
}

#[test]
fn test_complex_body_with_if_statement() {
    instrument();
    let bad_code = r"
def process [value] {
    if ($value > 10) {
        $value + 5
    } else {
        $value - 5
    }
}
";
    rule().assert_fix_contains(bad_code, "value: int");
}

#[test]
fn test_complex_body_with_nested_call() {
    let bad_code = r"
def transform [data] {
    $data | str trim | str upcase
}
";
    rule().assert_fix_contains(bad_code, "data: string");
}

#[test]
fn test_parameter_used_in_subexpression() {
    instrument();
    let bad_code = r"
def calculate [x] {
    let result = ($x + 10)
    $result
}
";
    rule().assert_fix_contains(bad_code, "x: int");
}

#[test]
fn test_parameter_with_closure() {
    let bad_code = r"
def apply [items] {
    $items | where {|x| $x > 5}
}
";
    rule().assert_fix_contains(bad_code, "items: list");
}

#[test]
fn test_parameter_with_field_access_in_closure() {
    let bad_code = r"
def get_names [people] {
    $people | each {|p| $p.name}
}
";
    rule().assert_fix_contains(bad_code, "people: list");
}

#[test]
fn test_nested_function_with_inference() {
    instrument();
    let bad_code = r#"
def outer [] {
    def inner [param] {
        $param | str trim
    }
    inner "test"
}
"#;
    rule().assert_fix_contains(bad_code, "param: string");
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
    rule().assert_fix_contains(bad_code, "text: string");
    rule().assert_fix_contains(bad_code, "items: list");
    rule().assert_fix_contains(bad_code, "count: int");
}

#[test]
fn test_param_used_in_comparison() {
    let bad_code = r"
def is_greater [value] {
    $value > 100
}
";
    rule().assert_fix_contains(bad_code, "value: int");
}

#[test]
fn test_param_in_binary_operation() {
    let bad_code = r"
def multiply [a, b] {
    $a * $b
}
";
    rule().assert_fix_contains(bad_code, "a: int");
    rule().assert_fix_contains(bad_code, "b: int");
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
    rule().assert_fix_contains(bad_code, "data: string");
    rule().assert_fix_contains(bad_code, "count: int");
    rule().assert_fix_contains(bad_code, "items: list");
}

#[test]
fn test_parameter_with_str_replace() {
    let bad_code = r#"
def clean [text] {
    $text | str replace "old" "new"
}
"#;
    rule().assert_fix_contains(bad_code, "text: string");
}

#[test]
fn test_parameter_with_str_downcase() {
    let bad_code = r"
def lowercase [input] {
    $input | str downcase
}
";
    rule().assert_fix_contains(bad_code, "input: string");
}

#[test]
fn test_parameter_with_str_upcase() {
    let bad_code = r"
def uppercase [input] {
    $input | str upcase
}
";
    rule().assert_fix_contains(bad_code, "input: string");
}

#[test]
fn test_parameter_with_filter() {
    let bad_code = r"
def filter_items [data] {
    $data | filter {|x| $x > 5}
}
";
    rule().assert_fix_contains(bad_code, "data: list");
}

#[test]
fn test_parameter_with_reduce() {
    let bad_code = r"
def sum_all [numbers] {
    $numbers | reduce {|it, acc| $acc + $it}
}
";
    rule().assert_fix_contains(bad_code, "numbers: list");
}

#[test]
fn test_parameter_with_append() {
    let bad_code = r"
def add_item [items] {
    $items | append 42
}
";
    rule().assert_fix_contains(bad_code, "items: list");
}

#[test]
fn test_parameter_with_prepend() {
    let bad_code = r"
def add_first [collection] {
    $collection | prepend 0
}
";
    rule().assert_fix_contains(bad_code, "collection: list");
}

#[test]
fn test_complex_body_with_let_statements() {
    instrument();
    let bad_code = r"
def compute [x] {
    let doubled = ($x * 2)
    let tripled = ($x * 3)
    $doubled + $tripled
}
";
    rule().assert_fix_contains(bad_code, "x: int");
}

#[test]
fn test_complex_body_with_multiple_pipelines() {
    instrument();
    let bad_code = r"
def process_data [data] {
    let cleaned = ($data | str trim)
    let upper = ($cleaned | str upcase)
    $upper
}
";
    rule().assert_fix_contains(bad_code, "data: string");
}

#[test]
fn test_exported_function_with_inference() {
    let bad_code = r"
export def process [text] {
    $text | str trim
}
";
    rule().assert_fix_contains(bad_code, "text: string");
}

#[test]
fn test_parameter_in_nested_block() {
    instrument();
    let bad_code = r"
def outer [value] {
    do {
        $value + 10
    }
}
";
    rule().assert_fix_contains(bad_code, "value: int");
}

#[test]
fn test_all_param_types_together() {
    instrument();
    let bad_code = r"
def complex [required, optional?, ...rest] {
    $required | str trim
    $optional | each { |x| $x }
    $rest | where {|x| $x > 0}
}
";
    rule().assert_fix_contains(bad_code, "required: string");
    rule().assert_fix_contains(bad_code, "optional?: list");
    rule().assert_fix_contains(bad_code, "...rest: list");
}

#[test]
fn test_deeply_nested_if_statements() {
    instrument();
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
    rule().assert_fix_contains(bad_code, "val: int");
}

#[test]
fn test_parameter_with_str_contains() {
    let bad_code = r#"
def has_word [text] {
    $text | str contains "word"
}
"#;
    rule().assert_fix_contains(bad_code, "text: string");
}
