use super::rule;

#[test]
fn detect_variable_capture_in_each() {
    let bad_code = r"
let items = [1, 2, 3]
let multiplier = 10
$items | each { $item * $multiplier }";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_variable_capture_in_where() {
    let bad_code = r"
let threshold = 5
$data | where { $value > $threshold }";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_temporary_variable_capture() {
    let bad_code = r"
let temp_config = load_config
$files | each { process_file $temp_config }";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_closure_with_outer_variable() {
    let bad_code = r"
def process_items [multiplier] {
    let items = [1, 2, 3]
    $items | each { $item * $multiplier }
}";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_nested_closure_capture() {
    let bad_code = r"
let config = get_config
$data | each {
    $row | each { process $item $config }
}";
    rule().assert_detects(bad_code);
}