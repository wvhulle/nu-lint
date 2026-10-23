use super::RULE;

#[test]
fn ignores_bare_variable() {
    let code = r#"let x = "hello"; echo $x"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_interpolation_with_text() {
    let code = r#"let x = "world"; echo $"hello ($x)""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_interpolation_with_multiple_variables() {
    let code = r#"let x = "a"; let y = "b"; echo $"($x) ($y)""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_plain_strings() {
    let code = r#"echo "hello""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_interpolation_with_expression() {
    let code = r#"echo $"(2 + 2)""#;
    RULE.assert_ignores(code);
}
