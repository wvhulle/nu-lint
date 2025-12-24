use super::RULE;

#[test]
fn ignores_http_get() {
    RULE.assert_ignores(r"http get https://example.com/file.tar.gz");
}

#[test]
fn ignores_http_get_with_save() {
    RULE.assert_ignores(r"http get https://example.com/file.tar.gz | save file.tar.gz");
}

#[test]
fn ignores_http_get_with_headers() {
    RULE.assert_ignores(r"http get --headers [Authorization Bearer] https://api.example.com");
}
