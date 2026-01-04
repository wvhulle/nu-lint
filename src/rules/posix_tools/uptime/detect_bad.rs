use super::RULE;

#[test]
fn test_detect_uptime() {
    RULE.assert_detects("^uptime");
}

#[test]
fn test_detect_uptime_with_flags() {
    RULE.assert_detects("^uptime -p");
}
