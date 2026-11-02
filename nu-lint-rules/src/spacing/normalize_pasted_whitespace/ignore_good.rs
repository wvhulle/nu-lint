use super::RULE;

#[test]
fn ignore_tab_in_strings() {
    RULE.assert_ignores("^echo 'first\tsecond'");
    RULE.assert_ignores("^echo \"first\u{a0}second\"");
    RULE.assert_ignores("^echo r#'first\tsecond'#");
}

#[test]
fn ignore_tab_in_string_inside_block() {
    RULE.assert_ignores("def build [] { ^echo 'first\tsecond' }");
}

#[test]
fn ignore_tab_in_comment() {
    RULE.assert_ignores("^echo first # some\tcomment");
}

#[test]
fn ignore_trailing_tab_handled_by_no_trailing_spaces() {
    RULE.assert_ignores("^echo first\t\n^echo second");
}
