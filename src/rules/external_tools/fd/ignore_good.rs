use super::RULE;

#[test]
fn ignores_nushell_ls_glob() {
    RULE.assert_ignores(r"ls **/*.rs");
}

#[test]
fn ignores_nushell_glob_command() {
    RULE.assert_ignores(r"glob **/*.rs");
}

#[test]
fn ignores_ls_with_where() {
    RULE.assert_ignores(r"ls | where type == file");
}
