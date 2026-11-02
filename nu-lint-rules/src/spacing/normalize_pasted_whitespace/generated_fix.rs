use super::RULE;

#[test]
fn fix_tab_between_tokens_to_one_space() {
    RULE.assert_fixed_is("^echo\t\tfirst", "^echo first");
}

#[test]
fn fix_tab_indentation_to_four_spaces() {
    RULE.assert_fixed_is("def build [] {\n\t^make\n}", "def build [] {\n    ^make\n}");
}

#[test]
fn fix_non_breaking_space_to_space() {
    RULE.assert_fixed_is("^echo\u{a0}first", "^echo first");
}
