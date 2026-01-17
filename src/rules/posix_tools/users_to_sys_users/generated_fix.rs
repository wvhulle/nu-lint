use super::RULE;

#[test]
fn test_fix_users() {
    RULE.assert_fixed_is("^users", "sys users | get user");
}
