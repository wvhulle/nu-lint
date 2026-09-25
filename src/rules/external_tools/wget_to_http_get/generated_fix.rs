use super::RULE;

#[test]
fn replaces_wget_url_with_http_get() {
    RULE.assert_fixed_is(
        "^wget https://example.com/file.tar.gz",
        "http get https://example.com/file.tar.gz",
    );
}

#[test]
fn replaces_short_output_flag_with_save() {
    RULE.assert_fixed_is(
        "^wget -O output.html https://example.com",
        "http get https://example.com | save output.html",
    );
}

#[test]
fn replaces_long_output_flag_with_save() {
    RULE.assert_fixed_is(
        "^wget --output-document file.txt https://example.com/data",
        "http get https://example.com/data | save file.txt",
    );
}

#[test]
fn no_fix_without_url() {
    RULE.assert_detects_without_fix("^wget");
    RULE.assert_detects_without_fix("^wget -O out.html");
}
