use super::rule;

#[test]
fn ignores_nushell_ls_glob() {
    rule().assert_ignores(r"ls **/*.rs");
}

#[test]
fn ignores_nushell_glob_command() {
    rule().assert_ignores(r"glob **/*.rs");
}

#[test]
fn ignores_ls_with_where() {
    rule().assert_ignores(r"ls | where type == file");
}
