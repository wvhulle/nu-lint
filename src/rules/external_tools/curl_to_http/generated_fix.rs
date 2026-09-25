use super::RULE;

#[test]
fn fix_simple_get_to_http_get() {
    RULE.assert_fixed_is(
        "^curl https://api.example.com",
        "http get https://api.example.com",
    );
}

#[test]
fn fix_explicit_get_method() {
    RULE.assert_fixed_is(
        "^curl -X GET https://api.example.com",
        "http get https://api.example.com",
    );
}

#[test]
fn fix_post_request() {
    RULE.assert_fixed_is(
        r#"^curl -X POST -d '{"key":"value"}' https://api.example.com"#,
        r#"http post https://api.example.com '{"key":"value"}'"#,
    );
}

#[test]
fn fix_put_request() {
    RULE.assert_fixed_is(
        r#"^curl -X PUT -d '{"updated":"data"}' https://api.example.com/resource"#,
        r#"http put https://api.example.com/resource '{"updated":"data"}'"#,
    );
}

#[test]
fn fix_delete_request() {
    RULE.assert_fixed_is(
        "^curl -X DELETE https://api.example.com/resource/123",
        "http delete https://api.example.com/resource/123",
    );
}

#[test]
fn fix_patch_request() {
    RULE.assert_fixed_is(
        r#"^curl -X PATCH -d '{"field":"patched"}' https://api.example.com/resource"#,
        r#"http patch https://api.example.com/resource '{"field":"patched"}'"#,
    );
}

#[test]
fn fix_header_conversion() {
    RULE.assert_fixed_is(
        "^curl -H 'Content-Type: application/json' https://api.example.com",
        "http get --headers [Content-Type application/json] https://api.example.com",
    );
}

#[test]
fn fix_multiple_headers() {
    RULE.assert_fixed_is(
        "^curl -H 'Accept: application/json' -H 'Authorization: Bearer token' https://api.example.com",
        "http get --headers [Accept application/json Authorization Bearer token] \
         https://api.example.com",
    );
}

#[test]
fn fix_user_password_auth() {
    RULE.assert_fixed_is(
        "^curl -u user:pass https://api.example.com",
        "http get --user user --password pass https://api.example.com",
    );
}

#[test]
fn fix_user_only_auth() {
    RULE.assert_fixed_is(
        "^curl -u username https://api.example.com",
        "http get --user username https://api.example.com",
    );
}

#[test]
fn fix_output_to_file() {
    RULE.assert_fixed_is(
        "^curl -o output.json https://api.example.com/data",
        "http get https://api.example.com/data | save output.json",
    );
}

#[test]
fn fix_data_implies_post() {
    RULE.assert_fixed_is(
        "^curl -d 'param=value' https://api.example.com",
        "http post https://api.example.com 'param=value'",
    );
}

#[test]
fn fix_data_raw() {
    RULE.assert_fixed_is(
        r#"^curl --data-raw '{"json":"data"}' https://api.example.com"#,
        r#"http post https://api.example.com '{"json":"data"}'"#,
    );
}

#[test]
fn fix_long_form_options() {
    RULE.assert_fixed_is(
        "^curl --request POST --header 'Content-Type: application/json' https://api.example.com",
        "http post --headers [Content-Type application/json] https://api.example.com",
    );
}

#[test]
fn no_fix_without_url() {
    RULE.assert_detects_without_fix("^curl");
    RULE.assert_detects_without_fix("^curl -X POST");
}
