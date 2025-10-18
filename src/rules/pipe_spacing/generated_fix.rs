#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::AstRule, rules::pipe_spacing::PipeSpacing};

    #[test]
    fn test_pipe_spacing_fix_no_spaces() {
        let rule = PipeSpacing;
        let bad_code = "echo 'hello'|str upcase";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect pipe without spaces");

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
            assert_eq!(
                fix.replacements[0].new_text, " | ",
                "Should replace with proper spacing"
            );
        });
    }

    #[test]
    fn test_pipe_spacing_fix_no_space_before() {
        let rule = PipeSpacing;
        let bad_code = "echo 'hello'| str upcase";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect pipe without space before"
            );

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text, " | ",
                "Should replace with proper spacing"
            );
        });
    }

    #[test]
    fn test_pipe_spacing_fix_no_space_after() {
        let rule = PipeSpacing;
        let bad_code = "echo 'hello' |str upcase";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect pipe without space after"
            );

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text, " | ",
                "Should replace with proper spacing"
            );
        });
    }

    #[test]
    fn test_pipe_spacing_fix_multiple_spaces() {
        let rule = PipeSpacing;
        let bad_code = "echo 'hello'  |  str upcase";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect pipe with multiple spaces"
            );

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text, " | ",
                "Should replace with proper spacing"
            );
        });
    }

    #[test]
    fn test_pipe_spacing_fix_description() {
        let rule = PipeSpacing;
        let bad_code = "echo 'hello'|str upcase";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect pattern");

            let violation = &violations[0];
            let fix = violation.fix.as_ref().unwrap();

            assert!(
                fix.description.contains("Fix pipe spacing"),
                "Fix description should mention pipe spacing"
            );
        });
    }
}
