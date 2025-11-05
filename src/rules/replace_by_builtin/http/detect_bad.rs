use crate::rules::replace_by_builtin::http::rule;

#[test]
fn detects_curl_get() {
    rule().assert_detects(r"^curl https://api.github.com/zen");
}

#[test]
fn detects_curl_with_headers() {
    let source = r"^curl -H 'Accept: application/json' https://api.github.com";
    rule().assert_fix_contains(source, "--headers");
    rule().assert_fix_contains(source, "Accept");
    rule().assert_fix_contains(source, "application/json");
}

#[test]
fn detects_curl_with_auth() {
    let source = r"^curl -u user:pass https://api.example.com";
    rule().assert_fix_contains(source, "--user user");
    rule().assert_fix_contains(source, "--password pass");
}

#[test]
fn detects_curl_post() {
    let source = r#"^curl -X POST -d '{"key":"value"}' https://api.example.com"#;
    rule().assert_fix_contains(source, "http post");
}

#[test]
fn detects_wget_download() {
    let source = r"^wget https://example.com/file.tar.gz";
    rule().assert_fix_contains(source, "http get");
    rule().assert_fix_description_contains(source, "save");
}

#[test]
fn detects_wget_with_output() {
    let source = r"^wget -O output.html https://example.com";
    rule().assert_fix_contains(source, "http get");
    rule().assert_fix_contains(source, "| save output.html");
}

#[test]
fn detects_fetch() {
    let source = r"^fetch https://api.example.com/data";
    rule().assert_fix_contains(source, "http get");
}

#[test]
fn fix_description_mentions_structured_data() {
    let source = r"^curl https://api.github.com";
    rule().assert_fix_description_contains(source, "structured");
}

#[test]
fn fix_description_mentions_pipeline_integration() {
    let source = r"^wget https://example.com/data.json";
    rule().assert_fix_description_contains(source, "pipeline");
}
