use super::RULE;

#[test]
fn test_detect_free_no_args() {
    RULE.assert_detects("^free");
}

#[test]
fn test_detect_free_human_readable() {
    RULE.assert_detects("^free -h");
}

#[test]
fn test_detect_free_megabytes() {
    RULE.assert_detects("^free -m");
}

#[test]
fn test_detect_free_gigabytes() {
    RULE.assert_detects("^free -g");
}

#[test]
fn test_detect_free_total() {
    RULE.assert_detects("^free -t");
}

#[test]
fn test_detect_free_multiple_flags() {
    RULE.assert_detects("^free -h -t");
}

#[test]
fn test_detect_free_long_flags() {
    RULE.assert_detects("^free --human --total");
}
