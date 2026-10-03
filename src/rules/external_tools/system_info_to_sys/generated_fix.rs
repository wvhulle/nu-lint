use super::RULE;

#[test]
fn fix_hostname() {
    RULE.assert_fixed_is("^hostname", "sys host | get hostname");
    RULE.assert_fixed_is("^uname -n", "sys host | get hostname");
}

#[test]
fn fix_kernel_release() {
    RULE.assert_fixed_is("^uname -r", "$nu.os-info.kernel_version");
}

#[test]
fn fix_machine() {
    RULE.assert_fixed_is("^uname --machine", "$nu.os-info.arch");
}

#[test]
fn fix_boot_time() {
    RULE.assert_fixed_is("^uptime -s", "sys host | get boot_time");
}

#[test]
fn fix_uptime_duration() {
    RULE.assert_fixed_is("^uptime -p", "sys host | get uptime");
}

#[test]
fn fix_memory() {
    RULE.assert_fixed_is("^free -b", "sys mem");
}

#[test]
fn fix_inside_subexpression() {
    RULE.assert_fixed_is(
        "let host = (^hostname)",
        "let host = (sys host | get hostname)",
    );
}
