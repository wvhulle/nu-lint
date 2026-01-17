use super::RULE;

#[test]
fn test_fix_uname_no_args() {
    RULE.assert_fixed_is("^uname", "sys host | get name");
}

#[test]
fn test_fix_uname_all() {
    RULE.assert_fixed_is("^uname -a", "sys host");
}

#[test]
fn test_fix_uname_kernel_release() {
    RULE.assert_fixed_is("^uname -r", "sys host | get kernel_version");
}
