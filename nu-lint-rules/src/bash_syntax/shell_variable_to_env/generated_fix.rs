use super::RULE;

#[test]
fn fix_bare_variable() {
    RULE.assert_fixed_is(
        "^echo $NOT_DEFINED_ANYWHERE",
        "^echo $env.NOT_DEFINED_ANYWHERE",
    );
}

#[test]
fn fix_braced_variable() {
    RULE.assert_fixed_is("^echo ${HOME}", "^echo $env.HOME");
}

#[test]
fn fix_last_exit_code() {
    RULE.assert_fixed_is("^echo $?", "^echo $env.LAST_EXIT_CODE");
}

#[test]
fn fix_variable_followed_by_path_to_interpolation() {
    RULE.assert_fixed_is("^ls $HOME/projects", "^ls $\"($env.HOME)/projects\"");
}
