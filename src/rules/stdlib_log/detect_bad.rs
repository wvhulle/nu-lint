use super::RULE;

#[test]
fn detect_custom_log_function() {
    RULE.assert_detects(r#"def log [msg] { print $msg }"#);
}

#[test]
fn detect_exported_custom_log_function() {
    RULE.assert_detects(r#"export def log [msg] { print $msg }"#);
}

#[test]
fn detect_log_debug_subcommand() {
    RULE.assert_detects(r#"def "log debug" [msg] { print $msg }"#);
}

#[test]
fn detect_log_debug_kebab() {
    RULE.assert_detects(r#"def log-debug [msg] { print $msg }"#);
}

#[test]
fn detect_log_info_subcommand() {
    RULE.assert_detects(r#"def "log info" [msg] { print $msg }"#);
}

#[test]
fn detect_log_info_kebab() {
    RULE.assert_detects(r#"def log-info [msg] { print $msg }"#);
}

#[test]
fn detect_log_warning_subcommand() {
    RULE.assert_detects(r#"def "log warning" [msg] { print $msg }"#);
}

#[test]
fn detect_log_warning_kebab() {
    RULE.assert_detects(r#"def log-warning [msg] { print $msg }"#);
}

#[test]
fn detect_log_error_subcommand() {
    RULE.assert_detects(r#"def "log error" [msg] { print $msg }"#);
}

#[test]
fn detect_log_error_kebab() {
    RULE.assert_detects(r#"def log-error [msg] { print $msg }"#);
}

#[test]
fn detect_log_critical_subcommand() {
    RULE.assert_detects(r#"def "log critical" [msg] { print $msg }"#);
}

#[test]
fn detect_log_critical_kebab() {
    RULE.assert_detects(r#"def log-critical [msg] { print $msg }"#);
}

#[test]
fn detect_multiple_custom_log_commands() {
    RULE.assert_count(
        r#"
def log [msg] { print $msg }
def "log info" [msg] { print $msg }
def log-error [msg] { print $msg }
"#,
        3,
    );
}
