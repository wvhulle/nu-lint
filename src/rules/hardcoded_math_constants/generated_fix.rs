use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_fix_hardcoded_pi() {
    init_env_log();
    let bad_code = r"
def calculate_circle_area [radius] {
    let pi = 3.14159
    $pi * $radius * $radius
}
";
    RULE.assert_fixed_contains(bad_code, "$math.PI");
}

#[test]
fn test_fix_hardcoded_e() {
    let bad_code = r"
def exponential_growth [time] {
    let e = 2.71828
    $e ** $time
}
";
    RULE.assert_fixed_contains(bad_code, "$math.E");
}

#[test]
fn test_fix_hardcoded_tau() {
    let bad_code = r"
def calculate_with_tau [] {
    let tau = 6.28318
    $tau / 2
}
";
    RULE.assert_fixed_contains(bad_code, "$math.TAU");
}

#[test]
fn test_fix_hardcoded_phi() {
    let bad_code = r"
def golden_ratio [] {
    1.618033
}
";
    RULE.assert_fixed_contains(bad_code, "$math.PHI");
}

#[test]
fn test_fix_hardcoded_gamma() {
    let bad_code = r"
def euler_mascheroni [] {
    0.577215
}
";
    RULE.assert_fixed_contains(bad_code, "$math.GAMMA");
}

#[test]
fn test_fix_pi_in_expression() {
    let bad_code = r"
def convert_degrees_to_radians [degrees] {
    $degrees * 3.14159 / 180
}
";
    RULE.assert_fixed_contains(bad_code, "$math.PI");
}

#[test]
fn test_fix_preserves_context() {
    let bad_code = r"
def circumference [radius] {
    2.0 * 3.14159 * $radius
}
";
    RULE.assert_fixed_contains(bad_code, "$math.PI");
    RULE.assert_fixed_contains(bad_code, "2.0");
    RULE.assert_fixed_contains(bad_code, "$radius");
}
