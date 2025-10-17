#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use crate::rules::snake_case_variables::SnakeCaseVariables;
    use crate::rule::Rule;

    #[test]
    fn test_invalid_snake_case() {
        let rule = SnakeCaseVariables::default();

        // These should trigger violations
        assert!(!SnakeCaseVariables::is_valid_snake_case("myVariable"));
        assert!(!SnakeCaseVariables::is_valid_snake_case("MyVariable"));
        assert!(!SnakeCaseVariables::is_valid_snake_case("MY_CONSTANT"));
    }

    #[test]
    fn test_bad_func_with_non_snake_case_variables() {
        let rule = SnakeCaseVariables::default();

        let bad_code = r#"
def bad-func [] {
    let myVariable = 5
    let AnotherVariable = 10
    let CamelCase = "bad"
}
"#;
        let context = LintContext::test_from_source(bad_code);
        let violations = rule.check(&context);
        assert!(
            violations.len() >= 3,
            "Should detect all non-snake_case variables, found {} violations",
            violations.len()
        );
    }
}