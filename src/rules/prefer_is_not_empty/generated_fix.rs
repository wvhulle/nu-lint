#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::AstRule, rules::prefer_is_not_empty::PreferIsNotEmpty};

    #[test]
    fn test_prefer_is_not_empty_fix_simple() {
        let rule = PreferIsNotEmpty;
        let bad_code = "if not ($list | is-empty) { echo 'has items' }";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect 'not ... is-empty' pattern");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
            assert_eq!(fix.replacements[0].new_text, "$list | is-not-empty", "Should convert to is-not-empty");
        });
    }

    #[test]
    fn test_prefer_is_not_empty_fix_variable() {
        let rule = PreferIsNotEmpty;
        let bad_code = "let has_data = not ($data | is-empty)";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect 'not ... is-empty' in assignment");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text, "$data | is-not-empty", "Should convert to is-not-empty");
        });
    }

    #[test]
    fn test_prefer_is_not_empty_fix_complex_expr() {
        let rule = PreferIsNotEmpty;
        let bad_code = "if not ($items | filter {|x| $x > 5} | is-empty) { echo 'found' }";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect complex expression with 'not ... is-empty'");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix for complex expression");

            let fix = violation.fix.as_ref().unwrap();
            assert!(fix.replacements[0].new_text.contains("| is-not-empty"), "Should end with is-not-empty");
            assert!(fix.replacements[0].new_text.contains("filter"), "Should preserve the filter");
        });
    }

    #[test]
    fn test_prefer_is_not_empty_fix_description() {
        let rule = PreferIsNotEmpty;
        let bad_code = "not ($list | is-empty)";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect pattern");

            let violation = &violations[0];
            let fix = violation.fix.as_ref().unwrap();

            assert!(fix.description.contains("Replace"), "Fix description should mention replacement");
            assert!(fix.description.contains("is-not-empty"), "Fix description should mention is-not-empty");
        });
    }

    #[test]
    fn test_prefer_is_not_empty_fix_multiple_patterns() {
        let rule = PreferIsNotEmpty;
        let bad_code = r#"
if not ($list | is-empty) and not ($other | is-empty) {
    echo "both not empty"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(violations.len(), 2, "Should detect both 'not ... is-empty' patterns");

            for violation in &violations {
                assert!(violation.fix.is_some(), "Should provide fix for each violation");
                let fix = violation.fix.as_ref().unwrap();
                assert!(fix.replacements[0].new_text.contains("| is-not-empty"), "Each fix should use is-not-empty");
            }
        });
    }
}