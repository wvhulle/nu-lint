#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::AstRule, rules::snake_case_variables::SnakeCaseVariables,
    };

    #[test]
    fn test_snake_case_fix_camel_case() {
        let rule = SnakeCaseVariables;
        let bad_code = "let myVariable = 5";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect camelCase variable");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
            assert_eq!(
                fix.replacements[0].new_text, "my_variable",
                "Should convert to snake_case"
            );
        });
    }

    #[test]
    fn test_snake_case_fix_pascal_case() {
        let rule = SnakeCaseVariables;
        let bad_code = "let MyVariable = 5";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect PascalCase variable");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text, "my_variable",
                "Should convert to snake_case"
            );
        });
    }

    #[test]
    fn test_snake_case_fix_mut_variable() {
        let rule = SnakeCaseVariables;
        let bad_code = "mut camelCase = 5";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect camelCase mutable variable"
            );

            let violation = &violations[0];
            assert!(
                violation.fix.is_some(),
                "Should provide a fix for mutable variable"
            );
            assert!(
                violation.message.contains("Mutable variable"),
                "Should identify as mutable variable"
            );

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text, "camel_case",
                "Should convert to snake_case"
            );
        });
    }

    #[test]
    fn test_snake_case_fix_multiple_variables() {
        let rule = SnakeCaseVariables;
        let bad_code = r"
let firstVar = 1
let secondVar = 2
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                2,
                "Should detect both camelCase variables"
            );

            for violation in &violations {
                assert!(
                    violation.fix.is_some(),
                    "Should provide fix for each violation"
                );
            }

            assert_eq!(
                violations[0].fix.as_ref().unwrap().replacements[0].new_text,
                "first_var"
            );
            assert_eq!(
                violations[1].fix.as_ref().unwrap().replacements[0].new_text,
                "second_var"
            );
        });
    }
}
