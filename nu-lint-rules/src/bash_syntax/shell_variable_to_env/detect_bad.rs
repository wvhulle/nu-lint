use super::RULE;

#[test]
fn detect_bare_variable() {
    RULE.assert_detects("^echo $NOT_DEFINED_ANYWHERE");
}

#[test]
fn detect_braced_variable() {
    RULE.assert_detects("^echo ${HOME}");
}

#[test]
fn detect_last_exit_code() {
    RULE.assert_detects("^echo $?");
}

#[test]
fn detect_variable_followed_by_path() {
    RULE.assert_detects("^ls $HOME/projects");
}

#[test]
fn detect_variable_after_shell_and_operator() {
    RULE.assert_detects("^make && ^echo $NOT_DEFINED_ANYWHERE");
}
