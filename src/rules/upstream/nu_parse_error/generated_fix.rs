use super::RULE;

#[test]
fn test_shell_andand_help_suggests_semicolon() {
    let code = "echo 'hello' && echo 'world'";
    RULE.assert_help_contains(code, "use ';' instead of the shell '&&'");
}

#[test]
fn test_shell_andand_help_suggests_and() {
    let code = "echo 'hello' && echo 'world'";
    RULE.assert_help_contains(code, "'and' instead of the boolean '&&'");
}

#[test]
fn test_unclosed_parenthesis_has_label() {
    let code = "let x = (";
    RULE.assert_labels_contain(code, "expected closing )");
}

#[test]
fn test_missing_positional_has_usage_help() {
    let code = "let";
    RULE.assert_help_contains(code, "Usage: let <var_name> = <initial_value>");
}

#[test]
fn test_unclosed_bracket_has_label() {
    let code = "let x = [1, 2, 3";
    RULE.assert_labels_contain(code, "expected closing ]");
}

#[test]
fn test_unclosed_brace_has_label() {
    let code = "def foo [] {";
    RULE.assert_labels_contain(code, "expected closing }");
}
