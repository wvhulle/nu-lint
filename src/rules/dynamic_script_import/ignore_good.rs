use super::RULE;

#[test]
fn test_static_use_literal_path() {
    let good_code = r#"use utils.nu"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_static_source_literal_path() {
    let good_code = r#"source config.nu"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_static_overlay_use_literal_path() {
    let good_code = r#"overlay use prompt.nu"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_static_use_with_quoted_string() {
    let good_code = r#"use "path/to/module.nu""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_use_stdlib_module() {
    let good_code = r#"use std/assert"#;
    RULE.assert_ignores(good_code);
}
