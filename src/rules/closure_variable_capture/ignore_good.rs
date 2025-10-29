use super::rule;

#[test]
fn ignore_proper_in_usage() {
    let good_code = r"
let items = [1, 2, 3]
$items | each { $in * 10 }";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_builtin_variables() {
    let good_code = r"
$env.PATH | each { $in | path exists }";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_closure_parameters() {
    let good_code = r"
def process_with_multiplier [multiplier] {
    { |item| $item * $multiplier }
}";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_proper_where_usage() {
    let good_code = r"
$data | where { $in.value > 5 }";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_it_variable() {
    let good_code = r"
$items | each { $it.name }";
    rule().assert_ignores(good_code);
}