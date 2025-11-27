use crate::rules::posix::cd::rule;

#[test]
fn ignores_builtin_cd() {
    let source = "cd";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_path() {
    let source = "cd /tmp";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_home() {
    let source = "cd ~";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_previous() {
    let source = "cd -";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_physical() {
    let source = "cd --physical /tmp";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_parent_levels() {
    let source = "cd ...";
    rule().assert_ignores(source);
}

#[test]
fn ignores_directory_path_alone() {
    let source = "/home";
    rule().assert_ignores(source);
}
