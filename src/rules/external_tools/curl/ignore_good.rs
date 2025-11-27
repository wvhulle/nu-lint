use super::rule;
use crate::log::instrument;

#[test]
fn ignores_http_get() {
    instrument();
    rule().assert_ignores(r"http get https://api.github.com");
}

#[test]
fn ignores_http_post() {
    instrument();
    rule().assert_ignores(r"http post https://api.example.com {key: 'value'}");
}

#[test]
fn ignores_http_put() {
    instrument();
    rule().assert_ignores(r"http put https://api.example.com/resource {data: 'updated'}");
}

#[test]
fn ignores_http_delete() {
    instrument();
    rule().assert_ignores(r"http delete https://api.example.com/resource/123");
}

#[test]
fn ignores_http_patch() {
    instrument();
    rule().assert_ignores(r"http patch https://api.example.com/resource {field: 'patched'}");
}

#[test]
fn ignores_http_head() {
    instrument();
    rule().assert_ignores(r"http head https://api.example.com");
}

#[test]
fn ignores_http_options() {
    instrument();
    rule().assert_ignores(r"http options https://api.example.com");
}

#[test]
fn ignores_http_with_headers() {
    instrument();
    rule().assert_ignores(
        r"http get --headers [Authorization 'Bearer token'] https://api.example.com",
    );
}

#[test]
fn ignores_http_with_user_auth() {
    instrument();
    rule().assert_ignores(r"http get --user username --password pass https://api.example.com");
}

#[test]
fn ignores_variable_command() {
    instrument();
    rule().assert_ignores(r"let cmd = 'curl'; ^$cmd https://example.com");
}
