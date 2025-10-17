use super::*;
use crate::rules::brace_spacing::BraceSpacing;

#[test]
fn test_good_brace_spacing() {
    let rule = BraceSpacing;

    let good = "{ key: value }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
