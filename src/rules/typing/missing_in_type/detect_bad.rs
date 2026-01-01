use super::RULE;

#[test]
fn detect_missing_input_type_annotation() {
    let bad_code = r"
def double [] {
    $in * 2
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_input_type_is_any() {
    let bad_code = r"
def double []: any -> int {
    $in * 2
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_missing_input_with_params() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_exported_function_missing_input_type() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_multiple_functions_missing_input_types() {
    let bad_code = r"
def first [] { $in | first }
def last [] { $in | last }
";
    RULE.assert_count(bad_code, 2);
}
