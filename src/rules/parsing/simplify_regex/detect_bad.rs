use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_detect_simple_regex_colon() {
    init_env_log();
    let bad_code = r#"'ip:port' | parse --regex '(?P<ip>.*):(?P<port>.*)'"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_simple_regex_space() {
    let bad_code = r#"'hello world' | parse --regex '(?P<first>.*) (?P<second>.*)'"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_simple_regex_at_sign() {
    let bad_code = r#"'user@domain' | parse --regex '(?P<user>.*)@(?P<domain>.*)'"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_simple_regex_equals() {
    let bad_code = r#"'key=value' | parse --regex '(?P<key>.*)=(?P<value>.*)'"#;
    RULE.assert_detects(bad_code);
}
