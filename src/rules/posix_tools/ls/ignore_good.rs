use crate::rules::posix_tools::ls::rule;

#[test]
fn ignores_builtin_ls() {
    let source = "ls";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_args() {
    let source = "ls --all *.rs";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_path() {
    let source = "ls /tmp";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_full_option() {
    let source = "ls --full-paths";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_pipe() {
    let source = "ls | where type == 'file'";
    rule().assert_ignores(source);
}
