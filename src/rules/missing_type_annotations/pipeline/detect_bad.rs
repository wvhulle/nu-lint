use super::rule;

#[test]
fn test_untyped_pipeline_input() {
    let bad_code = r"
def double [] {
    $in * 2
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_untyped_pipeline_output() {
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_untyped_both_input_output() {
    let bad_code = r"
def transform [] {
    $in | each { |x| $x + 1 }
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_exported_untyped_pipeline_input() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_untyped_with_parameters() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_multiple_untyped_commands() {
    let bad_code = r"
def first [] { $in | first }
def last [] { $in | last }
";
    rule().assert_violation_count_exact(bad_code, 4);
}
