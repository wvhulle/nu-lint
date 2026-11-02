use super::RULE;

#[test]
fn ignore_operator_text_in_strings() {
    RULE.assert_ignores(r#"^echo "make && make test""#);
    RULE.assert_ignores("$\"run (^pwd) && done\"");
}

#[test]
fn ignore_operator_text_in_comments() {
    RULE.assert_ignores("^make # && ^make test");
}

#[test]
fn ignore_nu_and_keyword() {
    RULE.assert_ignores("if true and false { ^make }");
}
