use crate::rules::replace_by_builtin::http::rule;

#[test]
fn ignores_http_get() {
    rule().assert_ignores(r"http get https://api.github.com/zen");
}

#[test]
fn ignores_http_post() {
    rule().assert_ignores(r"http post https://api.example.com {key: 'value'}");
}

#[test]
fn ignores_http_get_with_headers() {
    rule().assert_ignores(r"http get --headers [Accept 'application/json'] https://api.github.com");
}

#[test]
fn ignores_http_get_with_auth() {
    rule().assert_ignores(r"http get --user myuser --password mypass https://api.example.com");
}

#[test]
fn ignores_http_get_with_save() {
    rule().assert_ignores(r"http get https://example.com/file.tar.gz | save file.tar.gz");
}

#[test]
fn ignores_http_put() {
    rule().assert_ignores(r"http put https://api.example.com/resource {data: 'updated'}");
}

#[test]
fn ignores_http_patch() {
    rule().assert_ignores(r"http patch https://api.example.com/resource {field: 'new_value'}");
}

#[test]
fn ignores_http_delete() {
    rule().assert_ignores(r"http delete https://api.example.com/resource");
}

#[test]
fn ignores_http_head() {
    rule().assert_ignores(r"http head https://api.example.com");
}

#[test]
fn ignores_http_options() {
    rule().assert_ignores(r"http options https://api.example.com");
}
