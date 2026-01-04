use super::RULE;

#[test]
fn test_ignore_sys_disks() {
    RULE.assert_ignores("sys disks");
}

#[test]
fn test_ignore_sys_disks_with_where() {
    RULE.assert_ignores(r#"sys disks | where mount == "/home""#);
}

#[test]
fn test_ignore_unrelated_command() {
    RULE.assert_ignores("ls");
}
