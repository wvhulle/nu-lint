use super::RULE;

#[test]
fn test_detect_w() {
    RULE.assert_detects("^w");
}

#[test]
fn test_detect_w_with_user() {
    RULE.assert_detects("^w username");
}
