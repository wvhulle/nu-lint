use super::RULE;

#[test]
fn ignores_bare_words() {
    let code = r#"echo hello world"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_spaces() {
    let code = r#"echo "hello world""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_dash() {
    let code = r#"echo "-flag""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_dollar() {
    let code = r#"echo "$variable""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_pipe() {
    let code = r#"echo "a|b""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_interpolation() {
    let code = r#"let name = world; echo $"hello ($name)""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_backtick_strings() {
    let code = r#"ls `test`"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_raw_strings() {
    let code = r#"echo r#'test'#"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_empty_string() {
    let code = r#"echo """#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_number() {
    // Without quotes, "123" would be parsed as int
    let code = r#"echo "123""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_float() {
    // Without quotes, "3.14" would be parsed as float
    let code = r#"echo "3.14""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_true() {
    // Without quotes, "true" would be parsed as boolean
    let code = r#"echo "true""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_false() {
    // Without quotes, "false" would be parsed as boolean
    let code = r#"echo "false""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_null() {
    // Without quotes, "null" would be parsed as nothing
    let code = r#"echo "null""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_semicolon() {
    // Semicolon separates statements
    let code = r#"echo "a;b""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_single_quote() {
    // Single quote would start a quoted string
    let code = r#"echo "it's""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_hash() {
    // Hash starts a comment
    let code = "echo \"#comment\"";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_paren() {
    // Opening paren starts a subexpression
    let code = r#"echo "(1 + 2)""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_bracket() {
    // Opening bracket starts a list
    let code = r#"echo "[1, 2]""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_brace() {
    // Opening brace starts a record/closure
    let code = r#"echo "{a: 1}""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_double_ampersand() {
    // && is rejected by the parser
    let code = r#"echo "&&""#;
    RULE.assert_ignores(code);
}
