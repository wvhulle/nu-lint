use super::RULE;

#[test]
fn ignore_with_complete() {
    RULE.assert_ignores("^git clone https://example.com/repo | complete");
}

#[test]
fn ignore_curl_with_complete() {
    RULE.assert_ignores("^curl https://example.com | complete");
}

#[test]
fn ignore_complete_with_exit_check() {
    RULE.assert_ignores(
        r#"
        let result = ^git push origin main | complete
        if $result.exit_code != 0 {
            error make { msg: "git failed" }
        }
    "#,
    );
}

#[test]
fn ignore_git_status() {
    // git status is not in the LikelyErrors list - it's a read-only query
    RULE.assert_ignores("^git status");
}

#[test]
fn ignore_git_log() {
    // git log is also read-only
    RULE.assert_ignores("^git log --oneline");
}

#[test]
fn ignore_echo() {
    // echo doesn't have LikelyErrors effect
    RULE.assert_ignores("^echo hello");
}

#[test]
fn ignore_complete_in_function() {
    RULE.assert_ignores(
        r#"
        def safe-fetch [] {
            ^curl https://api.example.com | complete
        }
    "#,
    );
}
