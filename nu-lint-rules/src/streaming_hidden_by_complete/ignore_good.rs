use super::RULE;

#[test]
fn ignore_non_streaming_with_complete() {
    // curl without download flags is not streaming
    RULE.assert_ignores("^curl https://api.example.com | complete");
}

#[test]
fn ignore_streaming_without_complete() {
    // No complete, so nothing to warn about
    RULE.assert_ignores("^cargo build");
}

#[test]
fn ignore_git_status_complete() {
    // git status doesn't have streaming output
    RULE.assert_ignores("^git status | complete");
}

#[test]
fn ignore_git_log_complete() {
    // git log doesn't have streaming output
    RULE.assert_ignores("^git log | complete");
}

#[test]
fn ignore_jq_complete() {
    // jq is not a streaming command
    RULE.assert_ignores("^jq '.field' data.json | complete");
}

#[test]
fn ignore_streaming_piped_elsewhere() {
    // Piped to something else, not complete
    RULE.assert_ignores("^cargo build | lines | each { print $in }");
}
