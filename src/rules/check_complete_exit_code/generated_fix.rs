use super::rule;

// Note: This rule provides custom suggestions but does not provide automatic
// fixes. The suggestions are context-aware and include the specific variable
// name and command.

#[test]
fn test_suggestion_mentions_variable_name() {
    let bad_code = r"
let my_result = (^git status | complete)
";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "my_result");
    rule().assert_suggestion_contains(bad_code, "exit_code");
}

#[test]
fn test_suggestion_mentions_command_name() {
    let bad_code = r"
let result = (^make build | complete)
";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "make");
    rule().assert_suggestion_contains(bad_code, "result");
}

#[test]
fn test_suggestion_for_different_variable() {
    let bad_code = r"
mut fetch_output = (^curl https://example.com | complete)
";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "fetch_output");
    rule().assert_suggestion_contains(bad_code, "curl");
    rule().assert_suggestion_contains(bad_code, "exit_code");
}

#[test]
fn test_suggestion_shows_both_checking_styles() {
    let bad_code = r"
let result = (^make build | complete)
";

    rule().assert_suggestion_contains(bad_code, "if $result.exit_code");
    rule().assert_suggestion_contains(bad_code, "inline");
}

#[test]
fn test_violation_message_includes_command() {
    let bad_code = r"
let status = (^systemctl is-active bluetooth.service | complete)
";

    rule().assert_detects(bad_code);
    // The violation message should mention the command
    rule().assert_suggestion_contains(bad_code, "systemctl");
}
