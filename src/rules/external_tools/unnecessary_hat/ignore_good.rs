use super::RULE;

#[test]
fn ignores_external_without_hat() {
    RULE.assert_ignores("ddcutil setvcp 10");
}

#[test]
fn ignores_builtin_commands() {
    RULE.assert_ignores("ls");
}

#[test]
fn ignores_builtin_with_args() {
    RULE.assert_ignores("ls -la");
}

#[test]
fn ignores_pipeline_without_hat() {
    RULE.assert_ignores("git status | lines");
}

#[test]
fn ignores_function_definition() {
    RULE.assert_ignores(
        r#"
def my-command [] {
    echo "hello"
}
"#,
    );
}

#[test]
fn ignores_variable_assignment() {
    RULE.assert_ignores("let x = 10");
}

#[test]
fn ignores_if_statement() {
    RULE.assert_ignores("if true { echo yes }");
}

#[test]
fn ignores_for_loop() {
    RULE.assert_ignores("for i in 1..3 { echo $i }");
}

#[test]
fn ignores_hat_with_variable_args() {
    RULE.assert_ignores("let name = 'test'; ^echo $name");
}

#[test]
fn ignores_hat_in_pipeline() {
    RULE.assert_ignores("^ls | lines");
}
