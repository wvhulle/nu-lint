use super::RULE;

#[test]
fn test_ignore_using_std_math_constants() {
    let good_code = r"
use std/math

def calculate_circle_area [radius] {
    $math.PI * $radius * $radius
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_unrelated_numbers() {
    let good_code = r"
def calculate_price [quantity] {
    let price_per_unit = 3.14
    $quantity * $price_per_unit
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_similar_but_not_matching() {
    let good_code = r"
def some_value [] {
    3.2
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_low_precision_pi() {
    let good_code = r"
def rough_calculation [] {
    3.1
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_integers() {
    let good_code = r"
def integer_values [] {
    [1, 2, 3, 4, 5]
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_small_floats() {
    let good_code = r"
def small_values [] {
    [0.1, 0.2, 0.5]
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_already_using_math_pi() {
    let good_code = r"
use std/math

def circumference [radius] {
    2 * $math.PI * $radius
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_already_using_math_e() {
    let good_code = r"
use std/math

def exponential [x] {
    $math.E ** $x
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_unrelated_decimal() {
    let good_code = r#"
def version [] {
    "2.7.1"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_two_digit_precision_pi() {
    let good_code = r"
def approx [] {
    31.4
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_different_number() {
    let good_code = r"
def random_float [] {
    2.5
}
";
    RULE.assert_ignores(good_code);
}
