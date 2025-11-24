use super::rule;
use crate::log::instrument;

#[test]
fn detect_ripgrep_simple() {
    instrument();
    rule().assert_detects(r#"^rg \"pattern\""#);
}

#[test]
fn detect_ripgrep_with_file() {
    instrument();
    rule().assert_detects(r#"^rg \"error\" logs.txt"#);
}

#[test]
fn detect_ripgrep_with_flags() {
    instrument();
    rule().assert_detects(r#"^rg -i \"pattern\" file.txt"#);
}

#[test]
fn detect_ripgrep_invert_match() {
    instrument();
    rule().assert_detects(r#"^rg -v \"debug\" app.log"#);
}

#[test]
fn detect_ripgrep_line_numbers() {
    instrument();
    rule().assert_detects(r#"^rg -n \"TODO\" src/main.rs"#);
}

#[test]
fn detect_ripgrep_count() {
    instrument();
    rule().assert_detects(r#"^rg -c \"error\" logs.txt"#);
}

#[test]
fn detect_ripgrep_fixed_strings() {
    instrument();
    rule().assert_detects(r#"^rg -F \"literal\" README.md"#);
}

#[test]
fn detect_ripgrep_multiple_files() {
    instrument();
    rule().assert_detects(r#"^rg \"expr\" file1.nu file2.nu"#);
}

#[test]
fn detect_ripgrep_combined_flags() {
    instrument();
    rule().assert_detects(r#"^rg -nc \"panic\" src/main.rs"#);
}
