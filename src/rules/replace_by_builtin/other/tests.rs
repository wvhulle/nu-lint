use crate::rules::replace_by_builtin::other::rule;

#[test]
fn replaces_env_variable_access() {
    let source = "^printenv HOME";
    rule().assert_fix_contains(source, "$env.HOME");
    rule().assert_fix_description_contains(source, "directly");
}

#[test]
fn replaces_date_with_date_now() {
    let source = "^date";
    rule().assert_fix_contains(source, "date now");
    rule().assert_fix_description_contains(source, "datetime");
}

#[test]
fn replaces_hostname_with_sys_host() {
    let source = "^hostname";
    rule().assert_fix_contains(source, "(sys host).hostname");
    rule().assert_fix_description_contains(source, "sys");
}

#[test]
fn replaces_man_with_help() {
    let source = "^man ls";
    rule().assert_fix_contains(source, "help ls");
}

#[test]
fn replaces_which() {
    let source = "^which python";
    rule().assert_fix_contains(source, "which python");
}

#[test]
fn replaces_read_with_input() {
    let source = "^read";
    rule().assert_fix_contains(source, "input");
}

#[test]
fn replaces_read_silent_with_input_s() {
    let source = "^read -s";
    rule().assert_fix_contains(source, "input -s");
    rule().assert_fix_description_contains(source, "password");
}

#[test]
fn replaces_echo_with_print() {
    let source = "^echo hello";
    rule().assert_fix_contains(source, "print hello");
}

#[test]
fn replaces_wc_lines() {
    let source = "^wc -l";
    rule().assert_fix_contains(source, "lines | length");
    rule().assert_fix_description_contains(source, "count");
}

#[test]
fn replaces_awk_with_pipeline() {
    let source = "^awk";
    rule().assert_fix_contains(source, "where | select | each");
    rule().assert_fix_description_contains(source, "pipeline");
}

#[test]
fn replaces_cut_with_select() {
    let source = "^cut";
    rule().assert_fix_contains(source, "select");
    rule().assert_fix_description_contains(source, "columns");
}
