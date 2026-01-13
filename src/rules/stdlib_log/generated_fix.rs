use super::RULE;

#[test]
fn fix_removes_custom_log_function() {
    RULE.assert_fixed_is(r#"def log [msg] { print $msg }"#, "");
}

#[test]
fn fix_removes_custom_log_info_function() {
    RULE.assert_fixed_is(r#"def "log info" [msg] { print $msg }"#, "");
}

#[test]
fn fix_removes_custom_log_debug_kebab() {
    RULE.assert_fixed_is(r#"def log-debug [msg] { print $msg }"#, "");
}

#[test]
fn fix_preserves_other_code() {
    RULE.assert_fixed_is(
        r#"def my-func [] { 42 }
def log [msg] { print $msg }
def other-func [] { 1 }"#,
        r#"def my-func [] { 42 }
def other-func [] { 1 }"#,
    );
}

#[test]
fn fix_explanation_mentions_stdlib() {
    RULE.assert_fix_explanation_contains(r#"def log [msg] { print $msg }"#, "use std/log");
}
