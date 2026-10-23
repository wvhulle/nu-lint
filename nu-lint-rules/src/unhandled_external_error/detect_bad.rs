use super::RULE;

#[test]
fn detect_bare_git_commit() {
    // git commit can fail but doesn't have streaming output
    RULE.assert_detects("^git commit -m 'test'");
}

#[test]
fn detect_bare_git_add() {
    RULE.assert_detects("^git add .");
}

#[test]
fn detect_bare_curl() {
    RULE.assert_detects("^curl https://example.com");
}

#[test]
fn detect_in_pipeline_middle() {
    RULE.assert_detects("^curl https://example.com | lines | first");
}

#[test]
fn detect_in_function() {
    RULE.assert_detects(
        r#"
        def fetch-data [] {
            ^curl https://api.example.com
        }
    "#,
    );
}

#[test]
fn detect_in_closure() {
    RULE.assert_detects(
        r#"
        ["url1", "url2"] | each { |url| ^curl $url }
    "#,
    );
}

#[test]
fn detect_jq() {
    // jq can fail on invalid JSON but doesn't have streaming output
    RULE.assert_detects("^jq '.field' data.json");
}
