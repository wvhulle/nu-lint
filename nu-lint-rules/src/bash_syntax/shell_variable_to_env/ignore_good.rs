use super::RULE;

#[test]
fn ignore_defined_constant() {
    RULE.assert_ignores("const MAX_SIZE = 1\n^echo $MAX_SIZE");
}

#[test]
fn ignore_env_access() {
    RULE.assert_ignores("^echo $env.HOME");
}

#[test]
fn ignore_undefined_lowercase_variable() {
    RULE.assert_ignores("^echo $not_defined");
}

#[test]
fn ignore_variable_text_in_strings() {
    RULE.assert_ignores("^echo '$HOME'");
}

#[test]
fn ignore_defined_variable_inside_block_with_wider_parse_error() {
    RULE.assert_ignores(
        "def convert []: nothing -> string {\n    let INPUT_VALUE = 1\n    $INPUT_VALUE\n}",
    );
}
