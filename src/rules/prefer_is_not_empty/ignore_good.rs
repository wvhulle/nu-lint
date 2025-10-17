use super::*;

#[test]
fn test_is_not_empty_not_flagged() {
    let rule = PreferIsNotEmpty::default();

    let good_code = "if ($list | is-not-empty) { echo 'has items' }";
    let context = LintContext::test_from_source(good_code);
    assert_eq!(
        rule.check(&context).len(),
        0,
        "Should not flag 'is-not-empty'"
    );
}

#[test]
fn test_plain_is_empty_not_flagged() {
    let rule = PreferIsNotEmpty::default();

    let good_code = "if ($list | is-empty) { echo 'no items' }";
    let context = LintContext::test_from_source(good_code);
    assert_eq!(
        rule.check(&context).len(),
        0,
        "Should not flag plain 'is-empty'"
    );
}