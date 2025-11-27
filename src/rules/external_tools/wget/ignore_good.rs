use super::rule;

#[test]
fn ignores_http_get() {
    rule().assert_ignores(r"http get https://example.com/file.tar.gz");
}

#[test]
fn ignores_http_get_with_save() {
    rule().assert_ignores(r"http get https://example.com/file.tar.gz | save file.tar.gz");
}

#[test]
fn ignores_http_get_with_headers() {
    rule().assert_ignores(r"http get --headers [Authorization Bearer] https://api.example.com");
}
