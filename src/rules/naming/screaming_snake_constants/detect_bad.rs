use super::rule;

#[test]
fn test_detect_camel_case_constant() {
    let bad_code = "const maxValue = 100";

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_snake_case_constant() {
    let bad_code = "const my_constant = 200";

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_pascal_case_constant() {
    let bad_code = "const CamelCase = 300";

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}
