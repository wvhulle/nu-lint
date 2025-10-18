#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::RegexRule, rules::no_trailing_spaces::NoTrailingSpaces};

    #[test]
    fn ignores_no_trailing_spaces() {
        let code = "let x = 42";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(violations.is_empty());
        });
    }

    #[test]
    fn ignores_internal_spaces() {
        let code = "let x = 42 + 24";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(violations.is_empty());
        });
    }

    #[test]
    fn ignores_empty_lines() {
        let code = "let x = 42\n\nlet y = 43";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(violations.is_empty());
        });
    }

    #[test]
    fn ignores_proper_indentation() {
        let code = "def test [] {\n    let x = 42\n}";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(violations.is_empty());
        });
    }
}