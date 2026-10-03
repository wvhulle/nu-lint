use super::RULE;

#[test]
fn detect_scripted_curl_download() {
    for code in [
        "^curl -fsSL https://example.com/install.sh",
        "^curl -fL -o out.tar.gz https://example.com/a.tar.gz",
        "^curl --fail --location --silent https://example.com",
        "^curl -fsSL -H 'Accept: application/json' https://api.example.com",
        "^curl -fsSL -X GET https://example.com",
        "^curl -fsSLk --max-time 10 https://example.com",
    ] {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_wget_with_output_document() {
    RULE.assert_detects("^wget -qO- https://example.com");
    RULE.assert_detects("^wget -O page.html https://example.com");
}
