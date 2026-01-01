use super::RULE;

#[test]
fn ignore_properly_typed_input() {
    let good_code = r"
def double []: int -> int {
    $in * 2
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_function_without_pipeline_usage() {
    let good_code = r"
def add [a: int, b: int]: nothing -> int {
    $a + $b
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_function_not_using_in() {
    let good_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_specific_input_type_with_any_output() {
    let good_code = r"
def process []: string -> any {
    $in | str trim
}
";
    RULE.assert_ignores(good_code);
}
