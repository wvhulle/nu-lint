#[cfg(test)]
mod tests {
    
    use crate::{
        context::LintContext, rule::Rule, rules::screaming_snake_constants::ScreamingSnakeConstants,
    };

    #[test]
    fn test_detect_camel_case_constant() {
        let rule = ScreamingSnakeConstants;
        let bad_code = "const maxValue = 100";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect camelCase constant name"
            );
        });
    }

    #[test]
    fn test_detect_snake_case_constant() {
        let rule = ScreamingSnakeConstants;
        let bad_code = "const my_constant = 200";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect snake_case constant name"
            );
        });
    }

    #[test]
    fn test_detect_pascal_case_constant() {
        let rule = ScreamingSnakeConstants;
        let bad_code = "const CamelCase = 300";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect PascalCase constant name"
            );
        });
    }
}
