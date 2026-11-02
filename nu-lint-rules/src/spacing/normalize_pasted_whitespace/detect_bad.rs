use super::RULE;

#[test]
fn detect_tab_between_tokens() {
    RULE.assert_detects("^echo\tfirst");
}

#[test]
fn detect_tab_indentation() {
    RULE.assert_detects("def build [] {\n\t^make\n}");
}

#[test]
fn detect_non_breaking_space_in_bare_word() {
    RULE.assert_detects("^echo\u{a0}first");
}
