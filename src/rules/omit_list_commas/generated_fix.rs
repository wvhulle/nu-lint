#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::AstRule, rules::omit_list_commas::OmitListCommas};

    #[test]
    fn detects_violations_for_comma_in_list() {
        let rule = OmitListCommas;
        let code = "let items = [1, 2, 3]";

        LintContext::test_with_parsed_source(code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
            assert!(violations.iter().any(|v| v.message.contains("Omit commas")));
        });
    }
}