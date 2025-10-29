use crate::{config::Config, engine::LintEngine};

#[test]
fn test_unclosed_parenthesis() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = (";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations.is_empty(),
        "Expected parse error for unclosed parenthesis"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error")
    );
}

#[test]
fn test_unclosed_brace() {
    let engine = LintEngine::new(Config::default());
    let code = "def foo [] {";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations.is_empty(),
        "Expected parse error for unclosed brace"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error")
    );
}

#[test]
fn test_unclosed_bracket() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = [1, 2, 3";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations.is_empty(),
        "Expected parse error for unclosed bracket"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error")
    );
}

#[test]
fn test_unexpected_token() {
    let engine = LintEngine::new(Config::default());
    let code = "let let x = 5";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations.is_empty(),
        "Expected parse error for unexpected token"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error")
    );
}

#[test]
fn test_invalid_syntax() {
    let engine = LintEngine::new(Config::default());
    let code = "def [] { }";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations.is_empty(),
        "Expected parse error for missing function name"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error")
    );
}

#[test]
fn test_unclosed_string() {
    let engine = LintEngine::new(Config::default());
    let code = r#"let x = "unclosed string"#;
    let violations = engine.lint_source(code, None);

    assert!(
        !violations.is_empty(),
        "Expected parse error for unclosed string"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error")
    );
}

#[test]
fn test_multiple_parse_errors() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = (\nlet y = [";
    let violations = engine.lint_source(code, None);

    let parse_errors: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.as_ref() == "nu_parse_error")
        .collect();

    assert!(!parse_errors.is_empty(), "Expected at least one parse error");
}
