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

// Streaming output commands - users want to see live progress
#[test]
fn ignore_git_clone_streaming() {
    // git clone shows progress, buffering defeats the purpose
    RULE.assert_ignores("^git clone https://github.com/example/repo");
}

#[test]
fn ignore_git_push_streaming() {
    RULE.assert_ignores("^git push origin main");
}

#[test]
fn ignore_git_pull_streaming() {
    RULE.assert_ignores("^git pull");
}

#[test]
fn ignore_cargo_build_streaming() {
    // Build output should stream for progress visibility
    RULE.assert_ignores("^cargo build");
}

#[test]
fn ignore_cargo_test_streaming() {
    RULE.assert_ignores("^cargo test");
}

#[test]
fn ignore_npm_install_streaming() {
    RULE.assert_ignores("^npm install");
}

#[test]
fn ignore_make_streaming() {
    RULE.assert_ignores("^make");
}

#[test]
fn ignore_docker_build_streaming() {
    RULE.assert_ignores("^docker build .");
}

#[test]
fn ignore_wget_streaming() {
    // wget shows download progress by default
    RULE.assert_ignores("^wget https://example.com/file.tar.gz");
}
