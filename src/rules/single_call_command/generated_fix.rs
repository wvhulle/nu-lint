use super::RULE;

#[test]
fn fix_simple_no_params() {
    let code = "
def helper [] {
    42
}

def main [] {
    helper
}
";
    // Function body `42` replaces call site `helper`
    RULE.assert_fixed_contains(code, "    42\n}");
    // Definition is removed
    RULE.assert_fix_explanation_contains(code, "Inline function body");
}

#[test]
fn fix_with_single_param() {
    let code = "
def double [x: int] {
    $x * 2
}

def main [] {
    double 21
}
";
    // Parameter $x is substituted with argument 21
    RULE.assert_fixed_contains(code, "21 * 2");
}

#[test]
fn fix_with_multiple_params() {
    let code = "
def add [a b] {
    $a + $b
}

def main [] {
    add 1 2
}
";
    // Parameters $a and $b are substituted with arguments 1 and 2
    RULE.assert_fixed_contains(code, "1 + 2");
}

#[test]
fn fix_with_pipeline_input() {
    let code = "
def get_first [] {
    $in | first
}

def main [] {
    [1 2 3] | get_first
}
";
    // `$in | ` is stripped, leaving just `first`
    RULE.assert_fixed_contains(code, "[1 2 3] | first");
}

#[test]
fn fix_with_typed_parameter() {
    let code = "
def square [n: int] {
    $n * $n
}

def main [] {
    square 5
}
";
    // Parameter used multiple times - both occurrences substituted
    RULE.assert_fixed_contains(code, "5 * 5");
}

#[test]
fn fix_with_string_argument() {
    let code = r#"
def greet [name: string] {
    $"Hello, ($name)"
}

def main [] {
    greet "World"
}
"#;
    // String interpolation with parameter
    RULE.assert_fixed_contains(code, r#"$"Hello, ("World")""#);
}

#[test]
fn fix_with_list_argument() {
    let code = "
def sum_list [items] {
    $items | math sum
}

def main [] {
    sum_list [1 2 3]
}
";
    // Multi-element pipeline with list argument
    RULE.assert_fixed_contains(code, "[1 2 3] | math sum");
}

#[test]
fn fix_with_record_argument() {
    let code = "
def get_name [rec] {
    $rec.name
}

def main [] {
    get_name {name: bob}
}
";
    RULE.assert_fixed_contains(code, "{name: bob}.name");
}

#[test]
fn fix_with_expression_argument() {
    let code = "
def increment [x] {
    $x + 1
}

def main [] {
    increment (2 + 3)
}
";
    RULE.assert_fixed_contains(code, "(2 + 3) + 1");
}

#[test]
fn fix_with_variable_argument() {
    let code = "
def double [x] {
    $x * 2
}

def main [] {
    let val = 10
    double $val
}
";
    RULE.assert_fixed_contains(code, "$val * 2");
}

#[test]
fn fix_with_nested_function_call() {
    let code = "
def add_one [x] {
    $x + 1
}

def main [] {
    add_one (5 | into string | str length)
}
";
    RULE.assert_fixed_contains(code, "(5 | into string | str length) + 1");
}

#[test]
fn fix_pipeline_with_param() {
    let code = "
def take_n [n] {
    $in | take $n
}

def main [] {
    [1 2 3 4 5] | take_n 3
}
";
    // Pipeline input with parameter substitution
    RULE.assert_fixed_contains(code, "[1 2 3 4 5] | take 3");
}

#[test]
fn fix_with_closure_in_body() {
    let code = "
def filter_positive [] {
    $in | where {|x| $x > 0}
}

def main [] {
    [-1 0 1 2] | filter_positive
}
";
    RULE.assert_fixed_contains(code, "[-1 0 1 2] | where {|x| $x > 0}");
}

#[test]
fn fix_with_range_argument() {
    let code = "
def to_list [r] {
    $r | each {|x| $x}
}

def main [] {
    to_list 1..5
}
";
    // Range argument with pipeline
    RULE.assert_fixed_contains(code, "1..5 | each {|x| $x}");
}

#[test]
fn fix_with_command_substitution() {
    let code = "
def count_files [dir] {
    ls $dir | length
}

def main [] {
    count_files .
}
";
    // Command with pipeline
    RULE.assert_fixed_contains(code, "ls . | length");
}
