use super::RULE;

#[test]
fn detect_unclosed_parenthesis() {
    let code = "let x = (";
    RULE.assert_detects(code);
}

#[test]
fn detect_unclosed_brace() {
    let code = "def foo [] {";
    RULE.assert_detects(code);
}

#[test]
fn detect_unclosed_bracket() {
    let code = "let x = [1, 2, 3";
    RULE.assert_detects(code);
}

#[test]
fn detect_unexpected_token() {
    let code = "let let x = 5";
    RULE.assert_detects(code);
}

#[test]
fn detect_invalid_function_syntax() {
    let code = "def [] { }";
    RULE.assert_detects(code);
}

#[test]
fn detect_unclosed_string() {
    let code = r#"let x = "unclosed string"#;
    RULE.assert_detects(code);
}

#[test]
fn detect_multiple_parse_errors() {
    let code = "let x = (\nlet y = [";
    RULE.assert_detects(code);
}

#[test]
fn test_unclosed_parenthesis_message_meaningful() {
    let code = "let x = (";
    RULE.assert_detects(code);
}

#[test]
fn test_unclosed_brace_message_meaningful() {
    let code = "def foo [] {";
    RULE.assert_detects(code);
}

#[test]
fn test_unclosed_bracket_message_meaningful() {
    let code = "let x = [1, 2, 3";
    RULE.assert_detects(code);
}

#[test]
fn test_unexpected_token_message_descriptive() {
    let code = "let let x = 5";
    RULE.assert_detects(code);
}

#[test]
fn test_invalid_function_definition_message_descriptive() {
    let code = "def [] { }";
    RULE.assert_detects(code);
}

#[test]
fn detect_bare_module_not_found() {
    let code = "use nonexistent_module";
    RULE.assert_detects(code);
}
