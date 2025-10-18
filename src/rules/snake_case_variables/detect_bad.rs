#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::AstRule, rules::snake_case_variables::SnakeCaseVariables,
    };

    #[test]
    fn test_invalid_snake_case() {
        // These should trigger violations
        assert!(!SnakeCaseVariables::is_valid_snake_case("myVariable"));
        assert!(!SnakeCaseVariables::is_valid_snake_case("MyVariable"));
        assert!(!SnakeCaseVariables::is_valid_snake_case("MY_CONSTANT"));
        assert!(!SnakeCaseVariables::is_valid_snake_case("camelCase"));
        assert!(!SnakeCaseVariables::is_valid_snake_case("PascalCase"));
    }

    #[test]
    fn test_bad_let_variables() {
        let bad_code = r#"
def bad-func [] {
    let myVariable = 5
    let AnotherVariable = 10
    let CamelCase = "bad"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let rule = SnakeCaseVariables;
            let violations = rule.check(&context);
            assert!(
                violations.len() >= 3,
                "Should detect all non-snake_case let variables, found {} violations",
                violations.len()
            );
        });
    }

    #[test]
    fn test_bad_mut_variables() {
        let bad_code = "
def bad-func [] {
    mut MyCounter = 0
    mut totalSum = 100
    $MyCounter += 1
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let rule = SnakeCaseVariables;
            let violations = rule.check(&context);
            assert!(
                violations.len() >= 2,
                "Should detect non-snake_case mut variables, found {} violations",
                violations.len()
            );
        });
    }

    #[test]
    fn test_bad_shadowed_variables() {
        // Nushell allows variable shadowing - all shadowed variables should still use
        // snake_case
        let bad_code = "
def shadow-test [] {
    let myVar = 42
    let myVar = $myVar + 1
    let myVar = $myVar * 2
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let rule = SnakeCaseVariables;
            let violations = rule.check(&context);
            assert!(
                violations.len() >= 3,
                "Should detect all shadowed non-snake_case variables, found {} violations",
                violations.len()
            );
        });
    }

    #[test]
    fn test_bad_variables_with_various_naming_patterns() {
        let bad_code = r#"
def test-func [] {
    let HTTPRequest = "bad"
    let xmlParser = "bad"
    let URLEncoder = "bad"
    let firstName = "John"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let rule = SnakeCaseVariables;
            let violations = rule.check(&context);
            assert!(
                violations.len() >= 4,
                "Should detect various non-snake_case patterns, found {} violations",
                violations.len()
            );
        });
    }
}
