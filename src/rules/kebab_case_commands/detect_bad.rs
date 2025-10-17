#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use crate::rules::kebab_case_commands::KebabCaseCommands;
    use crate::rule::Rule;

    #[test]
    fn test_invalid_kebab_case() {
        assert!(!KebabCaseCommands::is_valid_kebab_case("myCommand"));
        assert!(!KebabCaseCommands::is_valid_kebab_case("my_command"));
    }

    #[test]
    fn test_detect_camel_case_command() {
        let rule = KebabCaseCommands::default();

        let bad_code = r#"
def myCommand [] {
    print "bad naming"
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect camelCase command name"
        );
    }

    #[test]
    fn test_detect_underscore_command() {
        let rule = KebabCaseCommands::default();

        let bad_code = r#"
def my_command [] {
    print "underscore instead of hyphen"
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect underscore in command name"
        );
    }

    #[test]
    fn test_detect_pascal_case_command() {
        let rule = KebabCaseCommands::default();

        let bad_code = r#"
def AnotherCommand [] {
    print "CamelCase"
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect PascalCase command name"
        );
    }
}