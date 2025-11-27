use super::rule;
use crate::log::instrument;

#[test]
fn fix_simple_get_to_http_get() {
    instrument();
    let source = r"^curl https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http get");
    rule().assert_replacement_contains(source, "https://api.example.com");
}

#[test]
fn fix_explicit_get_method() {
    instrument();
    let source = r"^curl -X GET https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http get");
}

#[test]
fn fix_post_request() {
    instrument();
    let source = r#"^curl -X POST -d '{"key":"value"}' https://api.example.com"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http post");
    rule().assert_replacement_contains(source, r#"'{"key":"value"}'"#);
}

#[test]
fn fix_put_request() {
    instrument();
    let source = r#"^curl -X PUT -d '{"updated":"data"}' https://api.example.com/resource"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http put");
}

#[test]
fn fix_delete_request() {
    instrument();
    let source = r"^curl -X DELETE https://api.example.com/resource/123";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http delete");
}

#[test]
fn fix_patch_request() {
    instrument();
    let source = r#"^curl -X PATCH -d '{"field":"patched"}' https://api.example.com/resource"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http patch");
}

#[test]
fn fix_header_conversion() {
    instrument();
    let source = r"^curl -H 'Content-Type: application/json' https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "--headers");
    rule().assert_replacement_contains(source, "Content-Type");
    rule().assert_replacement_contains(source, "application/json");
}

#[test]
fn fix_multiple_headers() {
    instrument();
    let source = r"^curl -H 'Accept: application/json' -H 'Authorization: Bearer token' https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "--headers");
    rule().assert_replacement_contains(source, "Accept");
    rule().assert_replacement_contains(source, "Authorization");
}

#[test]
fn fix_user_password_auth() {
    instrument();
    let source = r"^curl -u user:pass https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "--user user");
    rule().assert_replacement_contains(source, "--password pass");
}

#[test]
fn fix_user_only_auth() {
    instrument();
    let source = r"^curl -u username https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "--user username");
}

#[test]
fn fix_output_to_file() {
    instrument();
    let source = r"^curl -o output.json https://api.example.com/data";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "| save output.json");
}

#[test]
fn fix_data_implies_post() {
    instrument();
    let source = r"^curl -d 'param=value' https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http post");
}

#[test]
fn fix_data_raw() {
    instrument();
    let source = r#"^curl --data-raw '{"json":"data"}' https://api.example.com"#;
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http post");
    rule().assert_replacement_contains(source, r#"'{"json":"data"}'"#);
}

#[test]
fn fix_long_form_options() {
    instrument();
    let source =
        r"^curl --request POST --header 'Content-Type: application/json' https://api.example.com";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "http post");
    rule().assert_replacement_contains(source, "--headers");
}

#[test]
fn fix_explanation_mentions_method() {
    instrument();
    let source = r"^curl https://api.example.com";
    rule().assert_fix_explanation_contains(source, "Method:");
    rule().assert_fix_explanation_contains(source, "GET");
}

#[test]
fn fix_explanation_mentions_structured_data() {
    instrument();
    let source = r"^curl https://api.example.com";
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_explanation_mentions_benefits() {
    instrument();
    let source = r"^curl https://api.example.com";
    rule().assert_fix_explanation_contains(source, "Benefits:");
}

#[test]
fn fix_explanation_mentions_pipeline() {
    instrument();
    let source = r"^curl https://api.example.com";
    rule().assert_fix_explanation_contains(source, "pipeline");
}

#[test]
fn fix_explanation_describes_auth_conversion() {
    instrument();
    let source = r"^curl -u user:pass https://api.example.com";
    rule().assert_fix_explanation_contains(source, "Auth:");
}

#[test]
fn fix_explanation_describes_header_conversion() {
    instrument();
    let source = r"^curl -H 'Content-Type: application/json' https://api.example.com";
    rule().assert_fix_explanation_contains(source, "Headers:");
}

#[test]
fn fix_explanation_describes_data_conversion() {
    instrument();
    let source = r#"^curl -d '{"key":"value"}' https://api.example.com"#;
    rule().assert_fix_explanation_contains(source, "Data:");
}
