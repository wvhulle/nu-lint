use super::RULE;

#[test]
fn detect_bare_git_clone() {
    // git clone has LikelyErrors because it does network I/O and can fail
    RULE.assert_detects("^git clone https://github.com/example/repo");
}

#[test]
fn detect_bare_git_push() {
    RULE.assert_detects("^git push origin main");
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
fn detect_wget() {
    RULE.assert_detects("^wget https://example.com/file.tar.gz");
}
