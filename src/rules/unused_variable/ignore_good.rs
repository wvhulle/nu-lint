use super::RULE;

#[test]
fn test_used_variable() {
    RULE.assert_ignores("let x = 5; print $x");
}

#[test]
fn test_underscore_prefix() {
    RULE.assert_ignores("let _unused = 5");
}

#[test]
fn test_used_in_expression() {
    RULE.assert_ignores("let x = 5; $x + 1");
}

#[test]
fn test_used_in_condition() {
    RULE.assert_ignores("let x = true; if $x { print yes }");
}

#[test]
fn test_used_in_function_call() {
    RULE.assert_ignores("let name = 'world'; print $name");
}

#[test]
fn test_used_with_cell_path() {
    RULE.assert_ignores("let record = {a: 1}; $record.a");
}

#[test]
fn test_mut_used_and_reassigned() {
    let code = r#"
mut x = 0
$x = $x + 1
print $x
"#;
    RULE.assert_ignores(code);
}
