use super::RULE;

#[test]
fn ignore_nu_redirects() {
    RULE.assert_ignores("^make err> errors.txt");
    RULE.assert_ignores("^make out+err>| ^grep error");
}

#[test]
fn ignore_redirect_text_in_strings() {
    RULE.assert_ignores(r#"^echo "2>&1""#);
    RULE.assert_ignores("^echo '2>/dev/null'");
}

#[test]
fn ignore_redirect_text_in_comments() {
    RULE.assert_ignores("^make # 2>&1");
}
