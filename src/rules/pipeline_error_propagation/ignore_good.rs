use super::rule;

#[test]
fn ignore_pipeline_with_try() {
    rule().assert_ignores("try { ls | where type == file | get name | each { open $in } | to json }");
}

#[test]
fn ignore_pipeline_with_do_ignore() {
    rule().assert_ignores("do -i { ^git status | lines | where { $in | str contains 'modified' } }");
}

#[test]
fn ignore_short_pipeline() {
    rule().assert_ignores("ls | where type == file");
}

#[test]
fn ignore_pipeline_with_complete() {
    rule().assert_ignores("^git status | complete | get stdout | lines");
}

#[test]
fn ignore_pipeline_with_error_check() {
    rule().assert_ignores("let result = (^git status | complete); if $result.exit_code == 0 { $result.stdout | lines }");
}

#[test]
fn ignore_pipeline_with_error_make() {
    rule().assert_ignores("ls | get name | each { if not ($in | path exists) { error make { msg: 'File not found' } } }");
}

#[test]
fn ignore_safe_data_pipeline() {
    rule().assert_ignores("[1, 2, 3] | each { $in * 2 } | math sum");
}