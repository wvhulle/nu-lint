#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::RegexRule, rules::multiline_formatting::MultilineFormatting,
    };

    #[test]
    fn detects_violations_for_long_list() {
        let code = r#"let items = ["very", "long", "list", "with", "many", "items", "that", "should", "be", "multiline"]"#;

        LintContext::test_with_parsed_source(code, |context| {
            let rule = MultilineFormatting;
            let violations = rule.check(&context);
            assert!(!violations.is_empty());
            assert!(violations.iter().any(|v| v.message.contains("multi-line")));
        });
    }
}
