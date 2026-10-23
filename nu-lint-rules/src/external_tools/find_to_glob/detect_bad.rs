use super::RULE;

#[test]
fn detect_find_by_name_or_type() {
    for code in [
        "^find . -name '*.rs'",
        "^find src -type f",
        "^find . -type d -name target",
        r#"^find "my dir" -name "*.txt" -type f"#,
        "^find -name Cargo.toml",
    ] {
        RULE.assert_detects(code);
    }
}
