#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LintContext;
    use crate::rules::screaming_snake_constants::ScreamingSnakeConstants;
    use crate::rule::Rule;

    #[test]
    fn test_detect_camel_case_constant() {
        let rule = ScreamingSnakeConstants::default();

        let bad_code = "const maxValue = 100";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect camelCase constant name"
        );
    }

    #[test]
    fn test_detect_snake_case_constant() {
        let rule = ScreamingSnakeConstants::default();

        let bad_code = "const my_constant = 200";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect snake_case constant name"
        );
    }

    #[test]
    fn test_detect_pascal_case_constant() {
        let rule = ScreamingSnakeConstants::default();

        let bad_code = "const CamelCase = 300";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect PascalCase constant name"
        );
    }
}
