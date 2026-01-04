use super::RULE;

#[test]
fn test_ignore_sys_host() {
    RULE.assert_ignores("sys host");
}

#[test]
fn test_ignore_sys_host_with_get() {
    RULE.assert_ignores("sys host | get kernel_version");
}
