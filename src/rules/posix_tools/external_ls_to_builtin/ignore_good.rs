use super::RULE;

#[test]
fn ignores_builtin_ls() {
    let source = "ls";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_args() {
    let source = "ls --all *.rs";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_path() {
    let source = "ls /tmp";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_full_option() {
    let source = "ls --full-paths";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_pipe() {
    let source = "ls | where type == 'file'";
    RULE.assert_ignores(source);
}
