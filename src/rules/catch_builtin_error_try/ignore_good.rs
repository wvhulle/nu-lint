use super::RULE;

// HTTP commands in try blocks
#[test]
fn ignore_http_in_try_block() {
    RULE.assert_ignores("try { http get https://example.com }");
}

#[test]
fn ignore_http_with_try_catch() {
    RULE.assert_ignores("try { http get https://example.com } catch { |e| print $e }");
}

#[test]
fn ignore_http_in_nested_try() {
    RULE.assert_ignores(
        r#"
        def fetch [] {
            try {
                http get https://api.example.com
            }
        }
    "#,
    );
}

// File operations in try blocks
#[test]
fn ignore_open_in_try() {
    RULE.assert_ignores("try { open data.json }");
}

#[test]
fn ignore_save_in_try() {
    RULE.assert_ignores("try { { data: 1 } | save output.json }");
}

#[test]
fn ignore_rm_in_try() {
    RULE.assert_ignores("try { rm temp.txt }");
}

#[test]
fn ignore_cd_in_try() {
    RULE.assert_ignores("try { cd /some/path }");
}

#[test]
fn ignore_mkdir_in_try() {
    RULE.assert_ignores("try { mkdir new_dir }");
}

#[test]
fn ignore_from_json_in_try() {
    RULE.assert_ignores("try { $input | from json }");
}

// Commands that don't typically error at runtime
#[test]
fn ignore_ls_current_dir() {
    // ls without arguments lists current directory and rarely errors
    RULE.assert_ignores("ls");
}

#[test]
fn ignore_echo() {
    RULE.assert_ignores("echo hello");
}

#[test]
fn ignore_get_on_data() {
    RULE.assert_ignores("{ a: 1 } | get a");
}

#[test]
fn ignore_print() {
    RULE.assert_ignores("print hello");
}

#[test]
fn ignore_to_json() {
    // 'to' commands rarely error - serialization usually works
    RULE.assert_ignores("{ a: 1 } | to json");
}

#[test]
fn ignore_to_yaml() {
    RULE.assert_ignores("{ a: 1 } | to yaml");
}

// Commands with parse-time errors (can't be caught by try anyway)
#[test]
fn ignore_source() {
    // source errors at parse-time, not runtime - try can't catch it
    // This rule only concerns itself with runtime errors
    RULE.assert_ignores("source some_file.nu");
}

#[test]
fn ignore_source_env() {
    RULE.assert_ignores("source-env some_file.nu");
}

#[test]
fn ignore_hide() {
    // hide doesn't error when variable doesn't exist
    RULE.assert_ignores("hide foo");
}

#[test]
fn ignore_sleep() {
    // sleep type errors are caught at parse-time
    RULE.assert_ignores("sleep 1sec");
}

// exit cannot be caught by try - it terminates the process
#[test]
fn ignore_exit() {
    // exit bypasses try blocks entirely
    RULE.assert_ignores("exit 0");
}

#[test]
fn ignore_exit_nonzero() {
    // even non-zero exit can't be caught
    RULE.assert_ignores("exit 1");
}

// http without URL just shows help
#[test]
fn ignore_http_help() {
    RULE.assert_ignores("http");
}
