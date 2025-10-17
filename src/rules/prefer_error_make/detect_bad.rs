#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::RegexRule, rules::prefer_error_make::PreferErrorMake};

    #[test]
    fn test_detect_print_exit_pattern() {
        let rule = PreferErrorMake::new();
        let bad_code = r#"
def bad-error [] {
    print "Error occurred"
    exit 1
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(!rule.check(&context).is_empty());
        });
    }
}
