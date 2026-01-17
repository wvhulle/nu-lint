use super::RULE;

#[test]
fn detects_wget_simple_url() {
    RULE.assert_detects(r"^wget https://example.com/file.tar.gz");
}

#[test]
fn detects_wget_with_output_flag() {
    RULE.assert_detects(r"^wget -O output.html https://example.com");
}

#[test]
fn detects_wget_with_long_output_flag() {
    RULE.assert_detects(r"^wget --output-document=file.txt https://example.com");
}

#[test]
fn detects_wget_with_quiet_flag() {
    RULE.assert_detects(r"^wget -q https://example.com/data.json");
}
