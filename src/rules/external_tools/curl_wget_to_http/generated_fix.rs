use super::RULE;

#[test]
fn fix_curl_get() {
    RULE.assert_fixed_is(
        "^curl -fsSL https://example.com/install.sh",
        "http get --raw 'https://example.com/install.sh'",
    );
}

#[test]
fn fix_curl_keeps_quoted_url() {
    RULE.assert_fixed_is(
        r#"^curl -fsSL "https://example.com/a b?x=1&y=2""#,
        r#"http get --raw "https://example.com/a b?x=1&y=2""#,
    );
}

#[test]
fn fix_curl_variable_url() {
    RULE.assert_fixed_is(
        "let url = 'https://example.com'; ^curl -fsSL $url",
        "let url = 'https://example.com'; http get --raw $url",
    );
}

#[test]
fn fix_curl_output_file() {
    RULE.assert_fixed_is(
        "^curl -fL -o out.tar.gz https://example.com/a.tar.gz",
        "http get --raw 'https://example.com/a.tar.gz' | save --force out.tar.gz",
    );
}

#[test]
fn fix_curl_headers() {
    RULE.assert_fixed_is(
        "^curl -fsSL -H 'Accept: application/json' -H 'X-Token: abc' https://api.example.com",
        "http get --raw --headers ['Accept' 'application/json' 'X-Token' 'abc'] \
         'https://api.example.com'",
    );
}

#[test]
fn fix_curl_insecure_with_timeout() {
    RULE.assert_fixed_is(
        "^curl -fsSLk --max-time 10 https://example.com",
        "http get --raw --insecure --max-time 10sec 'https://example.com'",
    );
}

#[test]
fn fix_wget_to_stdout() {
    RULE.assert_fixed_is(
        "^wget -qO- https://example.com",
        "http get --raw 'https://example.com'",
    );
}

#[test]
fn fix_wget_to_file() {
    RULE.assert_fixed_is(
        "^wget -q -O page.html https://example.com",
        "http get --raw 'https://example.com' | save --force page.html",
    );
}
