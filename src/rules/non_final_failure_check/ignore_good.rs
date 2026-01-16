use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_already_using_complete() {
    init_env_log();
    let good_code = r"let result = (^curl https://api.example.com | complete)
if $result.exit_code != 0 { error make { msg: 'Failed' } }
$result.stdout | from json";
    RULE.assert_ignores(good_code);
}

#[test]
fn safe_git() {
    init_env_log();
    let good_code = r#"git config get remote.origin.url
    | str replace "git@ssh.gitgud.io:" "https://gitgud.io/"
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_single_external_command() {
    let good_code = r"^git status";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_safe_command() {
    let good_code = r"^echo 'test' | from json";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_bare_external_command_no_pipeline() {
    // Bare external commands without pipelines are not detected by this rule
    let good_code = r"^curl https://example.com";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_complete_in_subexpression() {
    let good_code =
        r"let data = (^curl https://api.example.com | complete | get stdout | from json)";
    RULE.assert_ignores(good_code);
}

#[test]
fn complete_stor() {
    init_env_log();
    RULE.assert_ignores("stor open | query db 'select'");
}

#[test]
fn test_external_with_ignore() {
    init_env_log();
    // Using `ignore` explicitly signals intentional discard of output and exit code
    let good_code = r"^curl https://api.example.com | ignore";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_external_with_ignore_in_pipeline() {
    init_env_log();
    // External piped to ignore should not trigger warning
    let good_code = r"^git fetch origin | ignore";
    RULE.assert_ignores(good_code);
}
