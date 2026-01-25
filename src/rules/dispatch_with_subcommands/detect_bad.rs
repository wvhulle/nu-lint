use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_detect_main_with_match_dispatch_three_branches() {
    init_test_log();
    let bad_code = r#"
def main [
    command?: string
] {
    match $command {
        "info" => { print "info" }
        "adjust" => { print "adjust" }
        "test" => { print "test" }
        _ => { print "Usage: ..." }
    }
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_main_with_match_dispatch_four_branches() {
    init_test_log();
    let bad_code = r#"
def main [
    cmd?: string
] {
    match $cmd {
        "start" => { run-start }
        "stop" => { run-stop }
        "restart" => { run-restart }
        "status" => { run-status }
        _ => { print "Unknown command" }
    }
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_main_with_default_string_param() {
    init_test_log();
    let bad_code = r#"
def main [
    action: string = "help"
] {
    match $action {
        "build" => { do-build }
        "test" => { do-test }
        "deploy" => { do-deploy }
        _ => { show-help }
    }
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_main_with_flags_and_dispatch() {
    init_test_log();
    let bad_code = r#"
def main [
    command?: string
    --verbose (-v)
] {
    match $command {
        "info" => { show-info $verbose }
        "debug" => { show-debug $verbose }
        "trace" => { show-trace $verbose }
        _ => { print "Usage" }
    }
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_only_one_violation_for_main() {
    init_test_log();
    let bad_code = r#"
def main [
    command?: string
] {
    match $command {
        "a" => { 1 }
        "b" => { 2 }
        "c" => { 3 }
        _ => { 0 }
    }
}
"#;

    RULE.assert_count(bad_code, 1);
}
