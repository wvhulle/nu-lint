#[cfg(test)]
mod tests {

    use crate::{context::LintContext, rule::Rule, rules::kebab_case_commands::KebabCaseCommands};

    #[test]
    fn test_invalid_kebab_case() {
        assert!(!KebabCaseCommands::is_valid_kebab_case("myCommand"));
        assert!(!KebabCaseCommands::is_valid_kebab_case("my_command"));
    }

    #[test]
    fn test_detect_camel_case_command() {
        let rule = KebabCaseCommands;

        let bad_code = r#"
def myCommand [] {
    print "bad naming"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect camelCase command name"
            );
        });
    }

    #[test]
    fn test_detect_underscore_command() {
        let rule = KebabCaseCommands;

        let bad_code = r#"
def my_command [] {
    print "underscore instead of hyphen"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect underscore in command name"
            );
        });
    }

    #[test]
    fn test_detect_pascal_case_command() {
        let rule = KebabCaseCommands;

        let bad_code = r#"
def AnotherCommand [] {
    print "CamelCase"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect PascalCase command name"
            );
        });
    }
}
