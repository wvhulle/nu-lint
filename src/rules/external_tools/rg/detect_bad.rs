use super::RULE;
use crate::log::init_env_log;

#[test]
fn detect_ripgrep_simple() {
    init_env_log();
    RULE.assert_detects(r#"^rg \"pattern\""#);
}

#[test]
fn detect_ripgrep_with_file() {
    init_env_log();
    RULE.assert_detects(r#"^rg \"error\" logs.txt"#);
}

#[test]
fn detect_ripgrep_with_flags() {
    init_env_log();
    RULE.assert_detects(r#"^rg -i \"pattern\" file.txt"#);
}

#[test]
fn detect_ripgrep_invert_match() {
    init_env_log();
    RULE.assert_detects(r#"^rg -v \"debug\" app.log"#);
}

#[test]
fn detect_ripgrep_line_numbers() {
    init_env_log();
    RULE.assert_detects(r#"^rg -n \"TODO\" src/main.rs"#);
}

#[test]
fn detect_ripgrep_count() {
    init_env_log();
    RULE.assert_detects(r#"^rg -c \"error\" logs.txt"#);
}

#[test]
fn detect_ripgrep_fixed_strings() {
    init_env_log();
    RULE.assert_detects(r#"^rg -F \"literal\" README.md"#);
}

#[test]
fn detect_ripgrep_multiple_files() {
    init_env_log();
    RULE.assert_detects(r#"^rg \"expr\" file1.nu file2.nu"#);
}

#[test]
fn detect_ripgrep_combined_flags() {
    init_env_log();
    RULE.assert_detects(r#"^rg -nc \"panic\" src/main.rs"#);
}
