use super::RULE;

#[test]
fn test_detect_simple_single_use_function() {
    RULE.assert_detects(
        r"
def helper [] {
    42
}

def main [] {
    helper
}
",
    );
}

#[test]
fn test_detect_single_use_function_with_parameter() {
    RULE.assert_detects(
        r"
def double [x: int] {
    $x * 2
}

def main [] {
    double 21
}
",
    );
}

#[test]
fn test_detect_single_use_function_with_pipeline() {
    RULE.assert_detects(
        r"
def get_first [] {
    $in | first
}

def main [] {
    [1 2 3] | get_first
}
",
    );
}

#[test]
fn test_detect_multiple_single_use_functions() {
    RULE.assert_count(
        r"
def add_one [x] {
    $x + 1
}

def double [x] {
    $x * 2
}

def main [] {
    let a = add_one 5
    let b = double 10
    $a + $b
}
",
        2,
    );
}

#[test]
fn test_detect_single_use_function_in_closure() {
    RULE.assert_detects(
        r"
def increment [] {
    $in + 1
}

def main [] {
    [1 2 3] | each { increment }
}
",
    );
}

#[test]
fn test_detect_single_use_function_with_string_processing() {
    RULE.assert_detects(
        r"
def uppercase_name [name: string] {
    str upcase $name
}

def main [] {
    uppercase_name 'alice'
}
",
    );
}

#[test]
fn test_detect_single_use_function_with_return_value() {
    RULE.assert_detects(
        r"
def calculate [] {
    5 + 3
}

def main [] {
    let result = calculate
    print $result
}
",
    );
}
