use super::rule;

#[test]
fn test_snake_case_fix_camel_case() {
    let bad_code = "let myVariable = 5";
    rule().assert_fix_contains(bad_code, "my_variable");
}

#[test]
fn test_snake_case_fix_pascal_case() {
    let bad_code = "let MyVariable = 5";
    rule().assert_fix_contains(bad_code, "my_variable");
}

#[test]
fn test_snake_case_fix_mut_variable() {
    let bad_code = "mut camelCase = 5";
    rule().assert_fix_contains(bad_code, "camel_case");
}

#[test]
fn test_snake_case_fix_multiple_variables() {
    let bad_code = r"
let firstVar = 1
let secondVar = 2
";
    rule().assert_violation_count_exact(bad_code, 2);
    rule().assert_fix_contains(bad_code, "first_var");
}
