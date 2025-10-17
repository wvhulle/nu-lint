#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_error_make::PreferErrorMake;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_print_exit_pattern() {
        let rule = PreferErrorMake::new();

        let bad_code = r#"
def bad-error [] {
    print "Error occurred"
    exit 1
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect print + exit pattern"
        );
    }

    #[test]
    fn test_detect_another_print_exit_pattern() {
        let rule = PreferErrorMake::new();

        let bad_code = r#"
def another-error [] {
    print "Something went wrong"
    exit 1
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect another print + exit pattern"
        );
    }
}
