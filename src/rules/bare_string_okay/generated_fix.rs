use super::RULE;

#[test]
fn fixes_double_quoted_simple_string() {
    let bad = r#"echo "hello""#;
    let good = "hello";
    assert_eq!(RULE.first_replacement_text(bad), good);
}

#[test]
fn fixes_single_quoted_simple_string() {
    let bad = r#"echo 'world'"#;
    let good = "world";
    assert_eq!(RULE.first_replacement_text(bad), good);
}

#[test]
fn fixes_quoted_url() {
    let bad = r#"http get "https://example.com""#;
    let good = "https://example.com";
    assert_eq!(RULE.first_replacement_text(bad), good);
}
