#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::RegexRule, rules::no_trailing_spaces::NoTrailingSpaces,
    };

    #[test]
    fn detects_trailing_spaces() {
        let code = "let x = 42   ";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
        });
    }

    #[test]
    fn detects_trailing_tabs() {
        let code = "let x = 42\t\t";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
        });
    }

    #[test]
    fn detects_mixed_trailing_whitespace() {
        let code = "let x = 42 \t ";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
        });
    }

    #[test]
    fn detects_multiple_lines_with_trailing_spaces() {
        let code = "let x = 42  \nlet y = 43   ";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = NoTrailingSpaces;
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
        });
    }
}
