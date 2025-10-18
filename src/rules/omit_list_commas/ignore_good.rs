#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::AstRule, rules::omit_list_commas::OmitListCommas};

    #[test]
    fn ignores_list_without_commas() {
        let rule = OmitListCommas;
        let code = "let items = [1 2 3]";

        LintContext::test_with_parsed_source(code, |context| {
            assert!(rule.check(&context).is_empty());
        });
    }

    #[test]
    fn ignores_empty_list() {
        let rule = OmitListCommas;
        let code = "let empty = []";

        LintContext::test_with_parsed_source(code, |context| {
            assert!(rule.check(&context).is_empty());
        });
    }

    #[test]
    fn ignores_single_item_list() {
        let rule = OmitListCommas;
        let code = "let single = [42]";

        LintContext::test_with_parsed_source(code, |context| {
            assert!(rule.check(&context).is_empty());
        });
    }

    #[test]
    fn ignores_multiline_list_without_commas() {
        let rule = OmitListCommas;
        let code = r#"let items = [
    "first"
    "second"
    "third"
]"#;

        LintContext::test_with_parsed_source(code, |context| {
            assert!(rule.check(&context).is_empty());
        });
    }
}