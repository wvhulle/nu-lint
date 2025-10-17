use super::*;
    use crate::rules::brace_spacing::BraceSpacing;

#[test]
fn test_good_brace_spacing() {
    let rule = BraceSpacing::default();

    let good = "{ key: value }";
    let context = LintContext::test_from_source(good);
    let violations = rule.check(&context);
    assert_eq!(violations.len(), 0);
}