use super::RULE;

#[test]
fn test_fix_w() {
    RULE.assert_fixed_is("^w", "sys users");
}
