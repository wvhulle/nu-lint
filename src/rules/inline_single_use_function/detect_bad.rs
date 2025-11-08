use super::rule;

#[test]
fn simple_single_use_single_line() {
    rule().assert_detects(
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
fn single_use_with_parameter() {
    rule().assert_detects(
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
fn single_use_with_pipeline() {
    rule().assert_detects(
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
fn multiple_single_use_functions() {
    rule().assert_violation_count(
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
fn single_use_in_closure() {
    rule().assert_detects(
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
fn single_use_string_operation() {
    rule().assert_detects(
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
fn single_use_with_return_value() {
    rule().assert_detects(
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
