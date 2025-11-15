use crate::rules::replace_by_builtin::other::rule;

#[test]
fn converts_printenv_to_env_variable_access() {
    let source = "^printenv HOME";
    rule().assert_fix_contains(source, "$env.HOME");
    rule().assert_fix_description_contains(source, "directly");
}

#[test]
fn converts_date_command_to_date_now() {
    let source = "^date";
    rule().assert_fix_contains(source, "date now");
    rule().assert_fix_description_contains(source, "datetime");
}

#[test]
fn converts_hostname_to_sys_host() {
    let source = "^hostname";
    rule().assert_fix_contains(source, "(sys host).hostname");
    rule().assert_fix_description_contains(source, "sys");
}

#[test]
fn converts_man_to_help() {
    let source = "^man ls";
    rule().assert_fix_contains(source, "help ls");
}

#[test]
fn converts_which_to_builtin_which() {
    let source = "^which python";
    rule().assert_fix_contains(source, "which python");
}

#[test]
fn converts_read_to_input() {
    let source = "^read";
    rule().assert_fix_contains(source, "input");
}

#[test]
fn converts_read_silent_to_input_secure() {
    let source = "^read -s";
    rule().assert_fix_contains(source, "input -s");
    rule().assert_fix_description_contains(source, "password");
}

#[test]
fn converts_echo_to_print() {
    let source = "^echo hello";
    rule().assert_fix_contains(source, "print hello");
}

#[test]
fn converts_wc_lines_to_lines_length() {
    let source = "^wc -l";
    rule().assert_fix_contains(source, "lines | length");
    rule().assert_fix_description_contains(source, "count");
}

#[test]
fn converts_awk_to_nu_pipeline() {
    let source = "^awk";
    rule().assert_fix_contains(source, "where | select | each");
    rule().assert_fix_description_contains(source, "pipeline");
}

#[test]
fn converts_cut_to_select() {
    let source = "^cut";
    rule().assert_fix_contains(source, "select");
    rule().assert_fix_description_contains(source, "columns");
}
