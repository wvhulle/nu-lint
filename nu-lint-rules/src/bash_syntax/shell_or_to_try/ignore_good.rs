use super::RULE;

#[test]
fn ignore_closure_with_empty_parameters() {
    RULE.assert_ignores("let greet = {|| ^echo hello }");
    RULE.assert_ignores("[1 2] | each {|| ^echo hello }");
    RULE.assert_ignores("[1 2] | each { || ^echo hello }");
}

#[test]
fn ignore_operator_text_in_strings_and_comments() {
    RULE.assert_ignores(r#"^echo "make || exit""#);
    RULE.assert_ignores("^make # || ^echo failed");
}

#[test]
fn ignore_try_catch() {
    RULE.assert_ignores("try { ^make } catch { ^echo failed }");
}
