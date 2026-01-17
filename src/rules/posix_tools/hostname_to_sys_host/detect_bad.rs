use super::RULE;

#[test]
fn test_detect_hostname() {
    RULE.assert_detects("^hostname");
}

#[test]
fn test_detect_hostname_with_flags() {
    RULE.assert_detects("^hostname -f");
}
