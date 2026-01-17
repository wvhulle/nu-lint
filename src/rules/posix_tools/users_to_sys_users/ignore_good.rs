use super::RULE;

#[test]
fn test_ignore_sys_users() {
    RULE.assert_ignores("sys users | get user");
}
