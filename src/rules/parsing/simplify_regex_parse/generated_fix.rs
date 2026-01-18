use super::RULE;

#[test]
fn test_fix_simple_regex_colon() {
    let bad_code = r#"'ip:port' | parse --regex '(?P<ip>.*):(?P<port>.*)'"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{ip}:{port}""#);
}

#[test]
fn test_fix_simple_regex_space() {
    let bad_code = r#"'hello world' | parse --regex '(?P<first>.*) (?P<second>.*)'"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{first} {second}""#);
}

#[test]
fn test_fix_simple_regex_at_sign() {
    let bad_code = r#"'user@domain' | parse --regex '(?P<user>.*)@(?P<domain>.*)'"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{user}@{domain}""#);
}

#[test]
fn test_fix_simple_regex_equals() {
    let bad_code = r#"'key=value' | parse --regex '(?P<key>.*)=(?P<value>.*)'"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{key}={value}""#);
}
