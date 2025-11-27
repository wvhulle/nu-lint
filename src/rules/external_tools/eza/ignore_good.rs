use super::rule;
use crate::log::instrument;

#[test]
fn ignores_nushell_ls() {
    instrument();
    rule().assert_ignores(r"ls");
}

#[test]
fn ignores_nushell_ls_with_all() {
    instrument();
    rule().assert_ignores(r"ls -a");
}

#[test]
fn ignores_nushell_ls_with_long() {
    instrument();
    rule().assert_ignores(r"ls -l");
}

#[test]
fn ignores_nushell_ls_with_glob() {
    instrument();
    rule().assert_ignores(r"ls **/*.rs");
}

#[test]
fn ignores_nushell_ls_with_where() {
    instrument();
    rule().assert_ignores(r"ls | where type == dir");
}

#[test]
fn ignores_nushell_ls_with_sort() {
    instrument();
    rule().assert_ignores(r"ls | sort-by size");
}

#[test]
fn ignores_nushell_ls_with_reverse() {
    instrument();
    rule().assert_ignores(r"ls | sort-by modified --reverse");
}

#[test]
fn ignores_variable_command() {
    instrument();
    rule().assert_ignores(r"let cmd = 'eza'; ^$cmd");
}

#[test]
fn ignores_glob_command() {
    instrument();
    rule().assert_ignores(r"glob **/*.rs");
}
