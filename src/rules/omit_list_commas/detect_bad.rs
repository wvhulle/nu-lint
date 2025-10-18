#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::AstRule, rules::omit_list_commas::OmitListCommas};

    #[test]
    fn detects_comma_in_list() {
        let rule = OmitListCommas;
        let code = "let items = [1, 2, 3]";

        LintContext::test_with_parsed_source(code, |context| {
            assert!(!rule.check(&context).is_empty());
        });
    }

    #[test]
    fn detects_multiple_commas_in_list() {
        let rule = OmitListCommas;
        let code = r#"let fruits = ["apple", "banana", "cherry"]"#;

        LintContext::test_with_parsed_source(code, |context| {
            assert!(!rule.check(&context).is_empty());
        });
    }

    #[test]
    fn detects_commas_in_nested_list() {
        let rule = OmitListCommas;
        let code = "let matrix = [[1, 2], [3, 4]]";

        LintContext::test_with_parsed_source(code, |context| {
            assert!(!rule.check(&context).is_empty());
        });
    }
}
