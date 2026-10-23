use super::RULE;

#[test]
fn test_is_not_empty_not_flagged() {
    let good_code = "if ($list | is-not-empty) { echo 'has items' }";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_plain_is_empty_not_flagged() {
    let good_code = "if ($list | is-empty) { echo 'no items' }";
    RULE.assert_ignores(good_code);
}
