use super::rule;
use crate::LintContext;

#[test]
fn test_bad_completion_naming_detected() {
    let bad_code = r"def complete-branches [] { ^git branch }";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert!(
            !violations.is_empty(),
            "Should detect bad completion function naming"
        );
    });
}
