use super::RULE;

#[test]
fn ignore_str_uppercase() {
    RULE.assert_ignores("'nu' | str uppercase");
}

#[test]
fn ignore_str_lowercase() {
    RULE.assert_ignores("'NU' | str lowercase");
}

#[test]
fn ignore_str_capitalize() {
    RULE.assert_ignores("'nu' | str capitalize");
}
