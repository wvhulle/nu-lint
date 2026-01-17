use super::RULE;

#[test]
fn test_fix_uptime() {
    RULE.assert_fixed_is("^uptime", "sys host | get uptime");
}
