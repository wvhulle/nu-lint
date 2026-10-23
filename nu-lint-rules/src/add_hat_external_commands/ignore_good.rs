use super::RULE;

#[test]
fn ignores_builtin_commands() {
    RULE.assert_ignores("ls");
}

#[test]
fn ignores_builtin_with_args() {
    RULE.assert_ignores("ls -la");
}

#[test]
fn ignores_variable_assignment() {
    RULE.assert_ignores("let x = 10");
}

#[test]
fn ignores_hat_in_pipeline() {
    RULE.assert_ignores("^ls | lines");
}

#[test]
fn ignores_hat_with_variable_command() {
    // When the command is a variable, the hat is necessary because the variable
    // might resolve to a builtin name at runtime
    RULE.assert_ignores("^git log");
}

#[test]
fn ignores_hat_with_env_variable_command() {
    // Environment variable as command - hat is required for safety
    RULE.assert_ignores("^$env.SHELL --version");
}
