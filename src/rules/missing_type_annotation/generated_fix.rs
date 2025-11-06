use super::rule;

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
