use super::RULE;

#[test]
fn test_ignore_sys_mem() {
    RULE.assert_ignores("sys mem");
}

#[test]
fn test_ignore_sys_mem_with_get() {
    RULE.assert_ignores("sys mem | get total");
}

#[test]
fn test_ignore_unrelated_command() {
    RULE.assert_ignores("ls");
}

#[test]
fn test_ignore_variable_named_free() {
    RULE.assert_ignores("let free = 100; print $free");
}
