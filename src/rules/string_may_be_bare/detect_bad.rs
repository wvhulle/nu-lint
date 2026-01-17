use super::RULE;

#[test]
fn detects_double_quoted_simple_string() {
    let code = r#"echo "hello""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_single_quoted_simple_string() {
    let code = r#"echo 'world'"#;
    RULE.assert_detects(code);
}

#[test]
fn detects_quoted_url() {
    let code = r#"http get "https://example.com""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_quoted_alphanumeric() {
    let code = r#"echo "abc123""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_path_starting_with_dot() {
    // Bare ./src/main.rs works in Nushell, so quotes are unnecessary
    let code = r#"echo "./src/main.rs""#;
    RULE.assert_detects(code);
}
