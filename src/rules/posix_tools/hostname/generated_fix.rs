use super::RULE;

#[test]
fn test_fix_hostname() {
    RULE.assert_fixed_is("^hostname", "sys host | get hostname");
}
