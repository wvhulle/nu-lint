use super::RULE;

#[test]
fn test_detect_who() {
    RULE.assert_detects("^who");
}

#[test]
fn test_detect_who_with_flags() {
    RULE.assert_detects("^who -a");
}
