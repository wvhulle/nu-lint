use super::RULE;

#[test]
fn test_detect_uname_no_args() {
    RULE.assert_detects("^uname");
}

#[test]
fn test_detect_uname_all() {
    RULE.assert_detects("^uname -a");
}

#[test]
fn test_detect_uname_kernel_release() {
    RULE.assert_detects("^uname -r");
}

#[test]
fn test_detect_uname_machine() {
    RULE.assert_detects("^uname -m");
}
