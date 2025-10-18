#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::RegexRule, rules::no_trailing_spaces::NoTrailingSpaces,
    };

    #[test]
    fn detects_violations_for_trailing_spaces() {
        let code = "let x = 42   ";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
            assert!(violations.iter().any(|v| v.message.contains("trailing")));
        });
    }
}
