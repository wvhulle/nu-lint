use super::RULE;

#[test]
fn ignore_get_optional() {
    RULE.assert_ignores("{a: 1} | get -o a");
}

#[test]
fn ignore_select_optional() {
    RULE.assert_ignores("[[a]; [1]] | select --optional a");
}

#[test]
fn ignore_get_ignore_case() {
    RULE.assert_ignores("{A: 1} | get --ignore-case a");
}
