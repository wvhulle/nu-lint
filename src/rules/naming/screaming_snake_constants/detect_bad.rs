use super::RULE;

#[test]
fn detect_camel_case_constant() {
    let bad_code = "const maxValue = 100";

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_snake_case_constant() {
    let bad_code = "const my_constant = 200";

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_pascal_case_constant() {
    let bad_code = "const CamelCase = 300";

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}
