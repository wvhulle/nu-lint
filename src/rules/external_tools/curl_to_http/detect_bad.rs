use super::RULE;
use crate::log::init_test_log;

#[test]
fn detects_curl_simple_url() {
    init_test_log();
    RULE.assert_detects(r"^curl https://example.com");
}

#[test]
fn detects_curl_get_explicit() {
    init_test_log();
    RULE.assert_detects(r"^curl -X GET https://api.github.com");
}

#[test]
fn detects_curl_post_with_data() {
    init_test_log();
    RULE.assert_detects(r#"^curl -X POST -d '{"name":"test"}' https://api.example.com"#);
}

#[test]
fn detects_curl_with_header() {
    init_test_log();
    RULE.assert_detects(r"^curl -H 'Authorization: Bearer token' https://api.example.com");
}

#[test]
fn detects_curl_with_multiple_headers() {
    init_test_log();
    RULE.assert_detects(
        r"^curl -H 'Accept: application/json' -H 'Content-Type: application/json' https://api.example.com",
    );
}

#[test]
fn detects_curl_with_user_auth() {
    init_test_log();
    RULE.assert_detects(r"^curl -u username:password https://api.example.com");
}

#[test]
fn detects_curl_with_user_only() {
    init_test_log();
    RULE.assert_detects(r"^curl -u username https://api.example.com");
}

#[test]
fn detects_curl_put_request() {
    init_test_log();
    RULE.assert_detects(
        r#"^curl -X PUT -d '{"updated":"value"}' https://api.example.com/resource"#,
    );
}

#[test]
fn detects_curl_delete_request() {
    init_test_log();
    RULE.assert_detects(r"^curl -X DELETE https://api.example.com/resource/123");
}

#[test]
fn detects_curl_patch_request() {
    init_test_log();
    RULE.assert_detects(r#"^curl -X PATCH -d '{"patch":"data"}' https://api.example.com/resource"#);
}

#[test]
fn detects_curl_with_output_file() {
    init_test_log();
    RULE.assert_detects(r"^curl -o output.json https://api.example.com/data");
}

#[test]
fn detects_curl_post_data_without_method() {
    init_test_log();
    RULE.assert_detects(r"^curl -d 'param=value' https://api.example.com");
}

#[test]
fn detects_curl_with_data_raw() {
    init_test_log();
    RULE.assert_detects(r#"^curl --data-raw '{"json":"data"}' https://api.example.com"#);
}

#[test]
fn detects_curl_long_form_options() {
    init_test_log();
    RULE.assert_detects(r#"^curl --request POST --header 'Content-Type: application/json' --data '{"test":1}' https://api.example.com"#);
}
