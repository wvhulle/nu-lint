use super::RULE;
use crate::log::init_test_log;

#[test]
fn ignores_http_get() {
    init_test_log();
    RULE.assert_ignores(r"http get https://api.github.com");
}

#[test]
fn ignores_http_post() {
    init_test_log();
    RULE.assert_ignores(r"http post https://api.example.com {key: 'value'}");
}

#[test]
fn ignores_http_put() {
    init_test_log();
    RULE.assert_ignores(r"http put https://api.example.com/resource {data: 'updated'}");
}

#[test]
fn ignores_http_delete() {
    init_test_log();
    RULE.assert_ignores(r"http delete https://api.example.com/resource/123");
}

#[test]
fn ignores_http_patch() {
    init_test_log();
    RULE.assert_ignores(r"http patch https://api.example.com/resource {field: 'patched'}");
}

#[test]
fn ignores_http_head() {
    init_test_log();
    RULE.assert_ignores(r"http head https://api.example.com");
}

#[test]
fn ignores_http_options() {
    init_test_log();
    RULE.assert_ignores(r"http options https://api.example.com");
}

#[test]
fn ignores_http_with_headers() {
    init_test_log();
    RULE.assert_ignores(
        r"http get --headers [Authorization 'Bearer token'] https://api.example.com",
    );
}

#[test]
fn ignores_http_with_user_auth() {
    init_test_log();
    RULE.assert_ignores(r"http get --user username --password pass https://api.example.com");
}

#[test]
fn ignores_variable_command() {
    init_test_log();
    RULE.assert_ignores(r"let cmd = 'curl'; ^$cmd https://example.com");
}
