use super::rule;

#[test]
fn test_fix_suggests_try_instead_of_do() {
    let bad_code = r"do { ^curl https://api.example.com }";

    rule().assert_help_contains(bad_code, "Replace 'do { ... }' with 'try { ... }'");
    rule().assert_help_contains(bad_code, "external commands");
    rule().assert_help_contains(bad_code, "error-prone operations");
}

#[test]
fn test_fix_mentions_error_handling_benefit() {
    let bad_code = r"do { open config.json | from json }";

    rule().assert_help_contains(bad_code, "proper error handling");
    rule().assert_help_contains(bad_code, "prevents script termination");
}

#[test]
fn test_fix_message_for_network_operations() {
    let bad_code = r#"do { http get "https://api.example.com" }"#;

    rule().assert_detects(bad_code);
    rule().assert_help_contains(bad_code, "network requests");
}
