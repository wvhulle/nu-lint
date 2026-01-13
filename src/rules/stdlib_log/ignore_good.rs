use super::RULE;

#[test]
fn ignore_unrelated_function() {
    RULE.assert_ignores(r#"def my-logger [msg] { print $msg }"#);
}

#[test]
fn ignore_when_stdlib_imported() {
    RULE.assert_ignores(
        r#"use std/log
def log [msg] { print $msg }"#,
    );
}

#[test]
fn ignore_log_debug_when_stdlib_imported() {
    RULE.assert_ignores(
        r#"use std/log
def "log debug" [msg] { print $msg }"#,
    );
}

#[test]
fn ignore_log_info_when_stdlib_imported() {
    RULE.assert_ignores(
        r#"use std/log
def "log info" [msg] { print $msg }"#,
    );
}

#[test]
fn ignore_log_warning_when_stdlib_imported() {
    RULE.assert_ignores(
        r#"use std/log
def "log warning" [msg] { print $msg }"#,
    );
}

#[test]
fn ignore_log_error_when_stdlib_imported() {
    RULE.assert_ignores(
        r#"use std/log
def "log error" [msg] { print $msg }"#,
    );
}

#[test]
fn ignore_log_critical_when_stdlib_imported() {
    RULE.assert_ignores(
        r#"use std/log
def "log critical" [msg] { print $msg }"#,
    );
}
