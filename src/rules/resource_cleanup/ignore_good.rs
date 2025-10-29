use super::rule;

#[test]
fn ignore_file_with_collect() {
    rule().assert_ignores("open large_file.txt | collect | lines | where { $in | str contains 'error' }");
}

#[test]
fn ignore_network_with_complete() {
    rule().assert_ignores("http get api.example.com/data | complete | get stdout | from json");
}

#[test]
fn ignore_external_process_with_complete() {
    rule().assert_ignores("^git log --oneline | complete | get stdout | lines");
}

#[test]
fn ignore_file_to_conversion() {
    rule().assert_ignores("open data.json | to nuon");
}

#[test]
fn ignore_file_into_conversion() {
    rule().assert_ignores("open data.csv | into json");
}

#[test]
fn ignore_file_save_operation() {
    rule().assert_ignores("open input.txt | lines | save output.txt");
}

#[test]
fn ignore_safe_external_commands() {
    rule().assert_ignores("^echo 'hello' | lines");
}

#[test]
fn ignore_wrapped_operations() {
    rule().assert_ignores("try { open risky_file.txt | lines | length }");
}