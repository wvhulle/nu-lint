use super::RULE;

// Note: This rule provides custom suggestions but does not provide automatic
// fixes. The suggestions are context-aware and include the specific variable
// name and command. This rule only triggers for dangerous external commands.

#[test]
fn test_suggestion_includes_actual_variable_name() {
    let bad_code = r"
let my_result = (^sed -i 's/foo/bar/g' file.txt | complete)
";

    RULE.assert_detects(bad_code);
    RULE.assert_help_contains(bad_code, "my_result");
    RULE.assert_help_contains(bad_code, "exit_code");
}

#[test]
fn test_suggestion_includes_external_command_name() {
    let bad_code = r"
let result = (^rm -rf /tmp/build | complete)
";

    RULE.assert_detects(bad_code);
    RULE.assert_help_contains(bad_code, "rm");
    RULE.assert_help_contains(bad_code, "result");
}

#[test]
fn test_suggestion_adapts_to_different_variable_names() {
    let bad_code = r"
mut fetch_output = (^sed -i '' config.txt | complete)
";

    RULE.assert_detects(bad_code);
    RULE.assert_help_contains(bad_code, "fetch_output");
    RULE.assert_help_contains(bad_code, "sed");
    RULE.assert_help_contains(bad_code, "exit_code");
}

#[test]
fn test_suggestion_provides_inline_and_separate_check_examples() {
    let bad_code = r"
let result = (^rm -rf /tmp/build | complete)
";

    RULE.assert_help_contains(bad_code, "if $result.exit_code");
    RULE.assert_help_contains(bad_code, "inline");
}

#[test]
fn test_violation_message_mentions_specific_external_command() {
    let bad_code = r"
let status = (^sed -i 's/x/y/g' service.txt | complete)
";

    RULE.assert_detects(bad_code);
    RULE.assert_help_contains(bad_code, "sed");
}
