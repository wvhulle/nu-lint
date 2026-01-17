use super::RULE;

#[test]
fn test_ignore_sys_host() {
    RULE.assert_ignores("sys host | get hostname");
}
