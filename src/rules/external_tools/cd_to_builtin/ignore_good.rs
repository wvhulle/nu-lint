use super::RULE;

#[test]
fn ignores_builtin_cd() {
    let source = "cd";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_path() {
    let source = "cd /tmp";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_home() {
    let source = "cd ~";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_previous() {
    let source = "cd -";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_physical() {
    let source = "cd --physical /tmp";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_cd_with_parent_levels() {
    let source = "cd ...";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_directory_path_alone() {
    let source = "/home";
    RULE.assert_ignores(source);
}
