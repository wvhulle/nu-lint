use super::RULE;

#[test]
fn ignore_backslash_in_middle_of_line() {
    RULE.assert_ignores("^echo \\ first");
}

#[test]
fn ignore_backslash_in_string() {
    RULE.assert_ignores("^echo '\\'\n^echo second");
}

#[test]
fn ignore_backslash_at_end_of_file() {
    RULE.assert_ignores("^echo first \\");
}

#[test]
fn ignore_backslash_followed_by_trailing_space() {
    RULE.assert_ignores("^echo first \\ \n^echo second");
}
