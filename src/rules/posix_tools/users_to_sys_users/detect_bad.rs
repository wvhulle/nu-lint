use super::RULE;

#[test]
fn test_detect_users() {
    RULE.assert_detects("^users");
}
