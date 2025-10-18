#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::AstRule, rules::kebab_case_commands::KebabCaseCommands};

    #[test]
    fn test_kebab_case_fix_camel_case() {
        let rule = KebabCaseCommands;
        let bad_code = "def myCommand [] { echo \"test\" }";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect camelCase command");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
            assert_eq!(fix.replacements[0].new_text, "my-command", "Should convert to kebab-case");
        });
    }

    #[test]
    fn test_kebab_case_fix_snake_case() {
        let rule = KebabCaseCommands;
        let bad_code = "def my_command [] { echo \"test\" }";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect snake_case command");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text, "my-command", "Should convert to kebab-case");
        });
    }

    #[test]
    fn test_kebab_case_fix_pascal_case() {
        let rule = KebabCaseCommands;
        let bad_code = "def MyCommand [] { echo \"test\" }";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect PascalCase command");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text, "my-command", "Should convert to kebab-case");
        });
    }

    #[test]
    fn test_kebab_case_fix_export_def() {
        let rule = KebabCaseCommands;
        let bad_code = "export def myCommand [] { echo \"test\" }";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect camelCase export command");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix for export def");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text, "my-command", "Should convert to kebab-case");
        });
    }

    #[test]
    fn test_kebab_case_fix_multiple_commands() {
        let rule = KebabCaseCommands;
        let bad_code = r#"
def firstCommand [] { echo "first" }
def secondCommand [] { echo "second" }
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(violations.len(), 2, "Should detect both camelCase commands");

            for violation in &violations {
                assert!(violation.fix.is_some(), "Should provide fix for each violation");
            }

            assert_eq!(violations[0].fix.as_ref().unwrap().replacements[0].new_text, "first-command");
            assert_eq!(violations[1].fix.as_ref().unwrap().replacements[0].new_text, "second-command");
        });
    }
}