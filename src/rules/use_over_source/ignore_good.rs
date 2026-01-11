use super::RULE;

#[test]
fn test_ignore_use_simple_module() {
    let good_code = r#"use std"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_use_with_path() {
    let good_code = r#"use ~/nushell/modules/utils.nu"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_use_relative_path() {
    let good_code = r#"use ./lib/helpers.nu"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_use_with_wildcard() {
    let good_code = r#"use std/log *"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_use_selective_import() {
    let good_code = r#"use std/math PI"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_use_multiple_imports() {
    let good_code = r#"use std/formats [ 'from ndjson' 'to ndjson' ]"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_use_in_function() {
    let good_code = r#"
def load_config [] {
    use config.nu
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_overlay_use() {
    let good_code = r#"overlay use spam"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_regular_code() {
    let good_code = r#"
def main [] {
    print "Hello World"
}
"#;
    RULE.assert_ignores(good_code);
}
