use super::rule;

#[test]
fn pure_function_only() {
    rule().assert_ignores(
        r"
def calculate [] {
    let x = 10
    let y = 20
    $x + $y
}

def main [] {
    calculate
}
",
    );
}

#[test]
fn side_effect_function_only() {
    rule().assert_ignores(
        r"
def display_message [] {
    print 'Hello'
    print 'World'
}

def main [] {
    display_message
}
",
    );
}

#[test]
fn properly_separated_functions() {
    rule().assert_ignores(
        r"
def calculate_sum [] {
    let numbers = 1..10
    $numbers | math sum
}

def display_result [] {
    let result = calculate_sum
    print $result
}

def main [] {
    display_result
}
",
    );
}

#[test]
fn single_pure_statement_before_side_effect() {
    rule().assert_ignores(
        r"
def quick_print [] {
    let msg = 'Hello'
    print $msg
}

def main [] {
    quick_print
}
",
    );
}

#[test]
fn main_function_not_checked() {
    rule().assert_ignores(
        r"
def main [] {
    let x = 10
    let y = 20
    let result = $x + $y
    print $result
}
",
    );
}

#[test]
fn function_with_only_control_flow() {
    rule().assert_ignores(
        r"
def conditional_logic [] {
    if true {
        print 'yes'
    } else {
        print 'no'
    }
}

def main [] {
    conditional_logic
}
",
    );
}

#[test]
fn very_short_function() {
    rule().assert_ignores(
        r"
def quick [] {
    print (10 + 20)
}

def main [] {
    quick
}
",
    );
}

#[test]
fn pure_pipeline_operations() {
    rule().assert_ignores(
        r"
def transform_data [] {
    [1 2 3 4 5]
    | where $it > 2
    | each { |x| $x * 2 }
    | math sum
}

def main [] {
    transform_data
}
",
    );
}

#[test]
fn exported_function_with_mixed_code() {
    rule().assert_ignores(
        r"
export def process [] {
    let x = 10
    let y = 20
    let result = $x + $y
    print $result
}

def main [] {
    process
}
",
    );
}

#[test]
fn function_without_main() {
    rule().assert_ignores(
        r"
def helper [] {
    let x = 10
    let y = 20
    let result = $x + $y
    print $result
}
",
    );
}

#[test]
fn side_effects_at_start() {
    rule().assert_ignores(
        r"
def process [] {
    print 'Starting'
    let x = 10
    let y = 20
    $x + $y
}

def main [] {
    process
}
",
    );
}

#[test]
fn alternating_pure_and_side_effects() {
    rule().assert_ignores(
        r"
def mixed [] {
    let x = 10
    print $x
    let y = 20
    print $y
}

def main [] {
    mixed
}
",
    );
}

#[test]
fn function_with_try_catch() {
    rule().assert_ignores(
        r"
def safe_operation [] {
    try {
        let data = [1 2 3]
        let result = ($data | math sum)
        print $result
    } catch {
        print 'Error occurred'
    }
}

def main [] {
    safe_operation
}
",
    );
}

#[test]
fn function_with_loop() {
    rule().assert_ignores(
        r"
def process_items [] {
    for item in [1 2 3] {
        let doubled = $item * 2
        print $doubled
    }
}

def main [] {
    process_items
}
",
    );
}

#[test]
fn function_with_while_loop() {
    rule().assert_ignores(
        r"
def count_down [] {
    mut counter = 5
    while $counter > 0 {
        print $counter
        $counter = $counter - 1
    }
}

def main [] {
    count_down
}
",
    );
}

#[test]
fn function_with_match() {
    rule().assert_ignores(
        r"
def check_value [x: int] {
    match $x {
        0 => { print 'zero' }
        1 => { print 'one' }
        _ => { print 'other' }
    }
}

def main [] {
    check_value 1
}
",
    );
}

#[test]
fn function_returning_pure_value() {
    rule().assert_ignores(
        r"
def get_value [] {
    let x = 10
    $x
}

def main [] {
    get_value
}
",
    );
}

#[test]
fn function_with_pipeline_input() {
    rule().assert_ignores(
        r"
def process_input [] {
    $in | where $it > 5 | each { |x| $x * 2 }
}

def main [] {
    [1 2 3 4 5 6] | process_input
}
",
    );
}

#[test]
fn function_with_closure_having_side_effects() {
    rule().assert_ignores(
        r"
def process_with_closure [] {
    [1 2 3] | each { |x| 
        let doubled = $x * 2
        print $doubled
    }
}

def main [] {
    process_with_closure
}
",
    );
}

#[test]
fn only_two_statements() {
    rule().assert_ignores(
        r"
def small_function [] {
    let x = 10
    print $x
}

def main [] {
    small_function
}
",
    );
}

#[test]
fn nested_if_with_side_effects() {
    rule().assert_ignores(
        r"
def nested_conditional [] {
    let value = 42
    if $value > 40 {
        if $value < 50 {
            print 'in range'
        }
    }
}

def main [] {
    nested_conditional
}
",
    );
}

#[test]
fn function_calling_pure_helpers() {
    rule().assert_ignores(
        r"
def add [a: int, b: int] {
    $a + $b
}

def multiply [a: int, b: int] {
    $a * $b
}

def calculate [] {
    let sum = (add 5 10)
    let product = (multiply 3 4)
    $sum + $product
}

def main [] {
    calculate
}
",
    );
}

#[test]
fn function_with_comments_only() {
    rule().assert_ignores(
        r"
def documented [] {
    # This is a comment
    # Another comment
    # More comments
    print 'Hello'
}

def main [] {
    documented
}
",
    );
}

#[test]
fn function_with_string_interpolation() {
    rule().assert_ignores(
        r"
def greet [name: string] {
    let greeting = $'Hello, ($name)!'
    print $greeting
}

def main [] {
    greet 'Alice'
}
",
    );
}

#[test]
fn function_with_record_creation() {
    rule().assert_ignores(
        r"
def create_record [] {
    {name: 'Alice', age: 30}
}

def main [] {
    create_record
}
",
    );
}

#[test]
fn function_with_list_operations() {
    rule().assert_ignores(
        r"
def filter_and_map [] {
    [1 2 3 4 5]
    | where $it > 2
    | each { |x| $x * 2 }
}

def main [] {
    filter_and_map
}
",
    );
}

#[test]
fn function_with_network_call_only() {
    rule().assert_ignores(
        r"
def fetch_data [] {
    http get 'https://api.example.com/data'
}

def main [] {
    fetch_data
}
",
    );
}

#[test]
fn function_with_file_read() {
    rule().assert_ignores(
        r"
def read_config [] {
    open config.toml
}

def main [] {
    read_config
}
",
    );
}

#[test]
fn exactly_two_pure_statements_no_side_effects() {
    rule().assert_ignores(
        r"
def two_statements [] {
    let x = 10
    let y = 20
}

def main [] {
    two_statements
}
",
    );
}

#[test]
fn pure_statements_with_pure_ending() {
    rule().assert_ignores(
        r"
def all_pure [] {
    let x = 10
    let y = 20
    let z = 30
    $x + $y + $z
}

def main [] {
    all_pure
}
",
    );
}
