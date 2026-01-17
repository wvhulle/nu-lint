use super::RULE;

#[test]
fn test_fix_who() {
    RULE.assert_fixed_is("^who", "sys users");
}
