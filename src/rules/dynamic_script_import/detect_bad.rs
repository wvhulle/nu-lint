use super::RULE;

#[test]
fn test_dynamic_overlay_use_with_variable() {
    let bad_code = r#"overlay use ($nu.data-dir | path join "vendor/autoload/oh-my-posh.nu")"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_dynamic_source_with_variable() {
    let bad_code = r#"source ($env.CONFIG_DIR | path join "config.nu")"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_dynamic_use_with_subexpression() {
    let bad_code = r#"use ([$nu.default-config-dir "utils.nu"] | path join)"#;
    RULE.assert_detects(bad_code);
}
