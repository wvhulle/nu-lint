use super::RULE;
use crate::log::instrument;

#[test]
fn test_help_contains_subcommand_suggestion() {
    instrument();
    let bad_code = r#"
def main [
    command?: string
] {
    match $command {
        "info" => { show-info }
        "adjust" => { run-adjust }
        "test" => { run-test }
        _ => { print "usage" }
    }
}
"#;

    RULE.assert_help_contains(bad_code, "def \"main info\"");
    RULE.assert_help_contains(bad_code, "def \"main adjust\"");
    RULE.assert_help_contains(bad_code, "def \"main test\"");
}

#[test]
fn test_help_mentions_automatic_help() {
    instrument();
    let bad_code = r#"
def main [
    cmd?: string
] {
    match $cmd {
        "start" => { 1 }
        "stop" => { 2 }
        "restart" => { 3 }
        _ => { 0 }
    }
}
"#;

    RULE.assert_help_contains(bad_code, "--help");
}

#[test]
fn test_violation_message_mentions_parameter_name() {
    instrument();
    let bad_code = r#"
def main [
    action?: string
] {
    match $action {
        "build" => { do-build }
        "test" => { do-test }
        "deploy" => { do-deploy }
        _ => { show-help }
    }
}
"#;

    RULE.assert_help_contains(bad_code, "def \"main build\"");
    RULE.assert_help_contains(bad_code, "def \"main deploy\"");
}
