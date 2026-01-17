use super::RULE;

#[test]
fn test_detect_source_simple_file() {
    let bad_code = r#"source common.nu"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_source_with_path() {
    let bad_code = r#"source ~/nushell/modules/utils.nu"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_source_relative_path() {
    let bad_code = r#"source ./lib/helpers.nu"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_source_in_function() {
    let bad_code = r#"
def load_config [] {
    source config.nu
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_source_with_string_interpolation() {
    let bad_code = r#"source $"($nu.default-config-dir)/common.nu""#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_source_in_main() {
    let bad_code = r#"
def main [] {
    source lib.nu
    print "Hello"
}
"#;
    RULE.assert_detects(bad_code);
}
