use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_detect_hardcoded_pi() {
    init_test_log();
    let bad_code = r"
def calculate_circle_area [radius] {
    let pi = 3.14159
    $pi * $radius * $radius
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_hardcoded_pi_high_precision() {
    let bad_code = r"
def precise_calculation [] {
    3.141592653589793
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_hardcoded_e() {
    let bad_code = r"
def exponential_growth [time] {
    let e = 2.71828
    $e ** $time
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_hardcoded_tau() {
    let bad_code = r"
def calculate_with_tau [] {
    let tau = 6.28318
    $tau / 2
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_hardcoded_phi() {
    let bad_code = r"
def golden_ratio [] {
    1.618033
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_hardcoded_gamma() {
    let bad_code = r"
def euler_mascheroni [] {
    0.577215
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_pi_in_expression() {
    let bad_code = r"
def convert_degrees_to_radians [degrees] {
    $degrees * 3.14159 / 180
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_constants() {
    let bad_code = r"
def math_operations [] {
    let pi = 3.14159
    let e = 2.71828
    [$pi, $e]
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_pi_in_calculation() {
    let bad_code = r"
def circumference [radius] {
    2.0 * 3.14159 * $radius
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_hardcoded_pi_as_negative() {
    let bad_code = r"
def negative_area [radius] {
    ($radius * $radius) * (-3.14159)
}
";
    RULE.assert_detects(bad_code);
}
