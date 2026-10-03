use super::RULE;

#[test]
fn ignore_builtin_input() {
    RULE.assert_ignores("let name = input 'Name: '");
    RULE.assert_ignores("let password = input --suppress-output");
}
