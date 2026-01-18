use super::RULE;

#[test]
fn replaces_wget_url_with_http_get() {
    let source = r"^wget https://example.com/file.tar.gz";
    RULE.assert_fixed_contains(source, "http get https://example.com/file.tar.gz");
}

#[test]
fn replaces_wget_output_with_save() {
    let source = r"^wget -O output.html https://example.com";
    RULE.assert_fixed_contains(source, "http get https://example.com | save output.html");
}

#[test]
fn replaces_wget_long_output_flag() {
    let source = r"^wget --output-document file.txt https://example.com/data";
    RULE.assert_fixed_contains(source, "http get https://example.com/data | save file.txt");
}

#[test]
fn preserves_url_in_replacement() {
    let source = r"^wget https://github.com/user/repo/archive/main.zip";
    RULE.assert_fixed_contains(
        source,
        "http get https://github.com/user/repo/archive/main.zip",
    );
}
