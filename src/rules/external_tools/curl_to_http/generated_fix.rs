use super::RULE;
use crate::log::init_env_log;

#[test]
fn fix_simple_get_to_http_get() {
    init_env_log();
    let source = r"^curl https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http get");
    RULE.assert_fixed_contains(source, "https://api.example.com");
}

#[test]
fn fix_explicit_get_method() {
    init_env_log();
    let source = r"^curl -X GET https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http get");
}

#[test]
fn fix_post_request() {
    init_env_log();
    let source = r#"^curl -X POST -d '{"key":"value"}' https://api.example.com"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http post");
    RULE.assert_fixed_contains(source, r#"'{"key":"value"}'"#);
}

#[test]
fn fix_put_request() {
    init_env_log();
    let source = r#"^curl -X PUT -d '{"updated":"data"}' https://api.example.com/resource"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http put");
}

#[test]
fn fix_delete_request() {
    init_env_log();
    let source = r"^curl -X DELETE https://api.example.com/resource/123";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http delete");
}

#[test]
fn fix_patch_request() {
    init_env_log();
    let source = r#"^curl -X PATCH -d '{"field":"patched"}' https://api.example.com/resource"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http patch");
}

#[test]
fn fix_header_conversion() {
    init_env_log();
    let source = r"^curl -H 'Content-Type: application/json' https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "--headers");
    RULE.assert_fixed_contains(source, "Content-Type");
    RULE.assert_fixed_contains(source, "application/json");
}

#[test]
fn fix_multiple_headers() {
    init_env_log();
    let source = r"^curl -H 'Accept: application/json' -H 'Authorization: Bearer token' https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "--headers");
    RULE.assert_fixed_contains(source, "Accept");
    RULE.assert_fixed_contains(source, "Authorization");
}

#[test]
fn fix_user_password_auth() {
    init_env_log();
    let source = r"^curl -u user:pass https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "--user user");
    RULE.assert_fixed_contains(source, "--password pass");
}

#[test]
fn fix_user_only_auth() {
    init_env_log();
    let source = r"^curl -u username https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "--user username");
}

#[test]
fn fix_output_to_file() {
    init_env_log();
    let source = r"^curl -o output.json https://api.example.com/data";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "| save output.json");
}

#[test]
fn fix_data_implies_post() {
    init_env_log();
    let source = r"^curl -d 'param=value' https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http post");
}

#[test]
fn fix_data_raw() {
    init_env_log();
    let source = r#"^curl --data-raw '{"json":"data"}' https://api.example.com"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http post");
    RULE.assert_fixed_contains(source, r#"'{"json":"data"}'"#);
}

#[test]
fn fix_long_form_options() {
    init_env_log();
    let source =
        r"^curl --request POST --header 'Content-Type: application/json' https://api.example.com";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "http post");
    RULE.assert_fixed_contains(source, "--headers");
}
