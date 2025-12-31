use super::RULE;
use crate::log::init_env_log;

#[test]
fn ignores_nushell_ls() {
    init_env_log();
    RULE.assert_ignores(r"ls");
}

#[test]
fn ignores_nushell_ls_with_all() {
    init_env_log();
    RULE.assert_ignores(r"ls -a");
}

#[test]
fn ignores_nushell_ls_with_long() {
    init_env_log();
    RULE.assert_ignores(r"ls -l");
}

#[test]
fn ignores_nushell_ls_with_glob() {
    init_env_log();
    RULE.assert_ignores(r"ls **/*.rs");
}

#[test]
fn ignores_nushell_ls_with_where() {
    init_env_log();
    RULE.assert_ignores(r"ls | where type == dir");
}

#[test]
fn ignores_nushell_ls_with_sort() {
    init_env_log();
    RULE.assert_ignores(r"ls | sort-by size");
}

#[test]
fn ignores_nushell_ls_with_reverse() {
    init_env_log();
    RULE.assert_ignores(r"ls | sort-by modified --reverse");
}

#[test]
fn ignores_variable_command() {
    init_env_log();
    RULE.assert_ignores(r"let cmd = 'eza'; ^$cmd");
}

#[test]
fn ignores_glob_command() {
    init_env_log();
    RULE.assert_ignores(r"glob **/*.rs");
}
