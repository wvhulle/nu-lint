use super::*;

#[test]
fn test_is_not_empty_not_flagged() {
    let rule = PreferIsNotEmpty;
    let good_code = "if ($list | is-not-empty) { echo 'has items' }";
    
    LintContext::test_with_parsed_source(good_code, |context| {
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag 'is-not-empty'"
        );
    });
}

#[test]
fn test_plain_is_empty_not_flagged() {
    let rule = PreferIsNotEmpty;
    let good_code = "if ($list | is-empty) { echo 'no items' }";
    
    LintContext::test_with_parsed_source(good_code, |context| {
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag plain 'is-empty'"
        );
    });
}
