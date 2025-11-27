use super::rule;

#[test]
fn replaces_wget_url_with_http_get() {
    let source = r"^wget https://example.com/file.tar.gz";
    rule().assert_replacement_contains(source, "http get https://example.com/file.tar.gz");
}

#[test]
fn replaces_wget_output_with_save() {
    let source = r"^wget -O output.html https://example.com";
    rule().assert_replacement_contains(source, "http get https://example.com | save output.html");
}

#[test]
fn replaces_wget_long_output_flag() {
    let source = r"^wget --output-document file.txt https://example.com/data";
    rule().assert_replacement_contains(source, "http get https://example.com/data | save file.txt");
}

#[test]
fn explains_save_for_downloads() {
    let source = r"^wget https://example.com/file.tar.gz";
    rule().assert_fix_explanation_contains(source, "save");
}

#[test]
fn explains_structured_data() {
    let source = r"^wget -O file.json https://api.example.com/data";
    rule().assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn preserves_url_in_replacement() {
    let source = r"^wget https://github.com/user/repo/archive/main.zip";
    rule().assert_replacement_contains(
        source,
        "http get https://github.com/user/repo/archive/main.zip",
    );
}
