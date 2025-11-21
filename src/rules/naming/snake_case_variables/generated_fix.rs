use super::rule;
use crate::{context::LintContext, fix::apply_fixes_to_stdin};

/// Helper function to apply fixes and get the fixed code
fn apply_fix(code: &str) -> String {
    let violations = LintContext::test_with_parsed_source(code, |context| {
        let mut violations = (rule().check)(&context);
        // Mark violations as coming from stdin for fix application
        for v in &mut violations {
            v.file = Some("<stdin>".into());
            v.source = Some(code.to_string().into());
        }
        violations
    });

    apply_fixes_to_stdin(&violations).unwrap_or_else(|| code.to_string())
}

#[test]
fn test_snake_case_fix_simple_usage() {
    let bad_code = "let myVariable = 5; print $myVariable";
    let fixed = apply_fix(bad_code);
    assert_eq!(fixed, "let my_variable = 5; print $my_variable");
}

#[test]
fn test_snake_case_fix_multiple_usages() {
    let bad_code = "let myVar = 5; print ($myVar + $myVar)";
    let fixed = apply_fix(bad_code);
    assert_eq!(fixed, "let my_var = 5; print ($my_var + $my_var)");
}

#[test]
fn test_snake_case_fix_mut_variable_with_usage() {
    let bad_code = "mut camelCase = 5; $camelCase = 10; print $camelCase";
    let fixed = apply_fix(bad_code);
    assert_eq!(
        fixed,
        "mut camel_case = 5; $camel_case = 10; print $camel_case"
    );
}

#[test]
fn test_snake_case_fix_in_expression() {
    let bad_code = "let myValue = 10; let result = $myValue * 2";
    let fixed = apply_fix(bad_code);
    assert_eq!(fixed, "let my_value = 10; let result = $my_value * 2");
}

#[test]
fn test_snake_case_fix_in_pipeline() {
    let bad_code = "let myList = [1, 2, 3]; $myList | each { |x| $x * 2 }";
    let fixed = apply_fix(bad_code);
    assert_eq!(
        fixed,
        "let my_list = [1, 2, 3]; $my_list | each { |x| $x * 2 }"
    );
}

#[test]
fn test_snake_case_fix_in_if_condition() {
    let bad_code = "let myFlag = true; if $myFlag { print 'yes' }";
    let fixed = apply_fix(bad_code);
    assert_eq!(fixed, "let my_flag = true; if $my_flag { print 'yes' }");
}

#[test]
fn test_snake_case_fix_multiple_variables() {
    let bad_code = r"let firstVar = 1; let secondVar = 2; print ($firstVar + $secondVar)";
    let fixed = apply_fix(bad_code);
    assert_eq!(
        fixed,
        "let first_var = 1; let second_var = 2; print ($first_var + $second_var)"
    );
}

#[test]
fn test_snake_case_fix_nested_scope() {
    let bad_code = r"let outerVar = 5; if true { print $outerVar }";
    let fixed = apply_fix(bad_code);
    assert_eq!(fixed, "let outer_var = 5; if true { print $outer_var }");
}

#[test]
fn test_snake_case_fix_with_cell_path() {
    let bad_code = "let myRecord = {a: 1}; print $myRecord.a";
    let fixed = apply_fix(bad_code);
    assert_eq!(fixed, "let my_record = {a: 1}; print $my_record.a");
}

#[test]
fn test_snake_case_fix_pascal_case() {
    let bad_code = "let MyVariable = 5; $MyVariable + 1";
    let fixed = apply_fix(bad_code);
    assert_eq!(fixed, "let my_variable = 5; $my_variable + 1");
}

#[test]
fn test_snake_case_no_change_for_correct_name() {
    let good_code = "let my_variable = 5; print $my_variable";
    let fixed = apply_fix(good_code);
    assert_eq!(fixed, good_code);
}
