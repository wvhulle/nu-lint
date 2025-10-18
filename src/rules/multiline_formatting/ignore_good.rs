#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::RegexRule, rules::multiline_formatting::MultilineFormatting};

    #[test]
    fn ignores_short_single_line_list() {
        let code = "let items = [1 2 3]";

        LintContext::test_with_parsed_source(code, |context| {
            let rule = MultilineFormatting;
            let violations = rule.check(&context);
            assert!(violations.is_empty());
        });
    }

    #[test]
    fn ignores_multiline_list() {
        let code = r#"let items = [
    "first"
    "second"
    "third"
]"#;

        LintContext::test_with_parsed_source(code, |context| {
            let rule = MultilineFormatting;
            let violations = rule.check(&context);
            assert!(violations.is_empty());
        });
    }

    #[test]
    fn ignores_multiline_function() {
        let code = r#"def process_data [
    input: string
    output: string
] {
    echo "processing"
}"#;

        LintContext::test_with_parsed_source(code, |context| {
            let rule = MultilineFormatting;
            let violations = rule.check(&context);
            assert!(violations.is_empty());
        });
    }
}