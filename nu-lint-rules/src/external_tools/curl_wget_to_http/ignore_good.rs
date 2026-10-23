use super::RULE;

#[test]
fn ignore_curl_without_fail_and_location() {
    RULE.assert_ignores("^curl -s https://example.com");
    RULE.assert_ignores("^curl -sL https://example.com");
    RULE.assert_ignores("^curl -fs https://example.com");
}

#[test]
fn ignore_curl_with_other_requests() {
    for code in [
        "^curl -fsSL -X POST -d 'a=1' https://example.com",
        "^curl -fsSL -d @body.json https://example.com",
        "^curl -fsSL -u user:pass https://example.com",
        "^curl -fsSL -I https://example.com",
        "^curl -fsSL -O https://example.com/a.tar.gz",
        "^curl -fsSL -A agent https://example.com",
        "^curl -fsSL https://a.example.com https://b.example.com",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_exit_code_checks() {
    RULE.assert_ignores("^curl -fsSL https://example.com | complete");
}

#[test]
fn ignore_wget_saving_by_url_name() {
    RULE.assert_ignores("^wget https://example.com/a.tar.gz");
    RULE.assert_ignores("^wget -r https://example.com");
    RULE.assert_ignores("^wget -P /tmp https://example.com/a.tar.gz");
}

#[test]
fn ignore_builtin_http() {
    RULE.assert_ignores("http get https://example.com");
}
