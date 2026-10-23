use super::RULE;

#[test]
fn ignore_plain_get() {
    RULE.assert_ignores("{a: 1} | get a");
}

#[test]
fn ignore_case_insensitive_member() {
    RULE.assert_ignores("{A: 1} | get a!");
}
