use crate::{config::Config, engine::LintEngine};

#[test]
fn test_valid_let_statement() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = 5";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid let statement"
    );
}

#[test]
fn test_valid_function_definition() {
    let engine = LintEngine::new(Config::default());
    let code = r#"
def greet [name: string] {
    print $"Hello, ($name)!"
}
"#;
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid function"
    );
}

#[test]
fn test_valid_pipeline() {
    let engine = LintEngine::new(Config::default());
    let code = "ls | where size > 100 | sort-by name";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid pipeline"
    );
}

#[test]
fn test_valid_list() {
    let engine = LintEngine::new(Config::default());
    let code = "let items = [1, 2, 3, 4, 5]";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid list"
    );
}

#[test]
fn test_valid_record() {
    let engine = LintEngine::new(Config::default());
    let code = "let person = {name: 'John', age: 30}";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid record"
    );
}

#[test]
fn test_valid_string() {
    let engine = LintEngine::new(Config::default());
    let code = r#"let greeting = "Hello, world!""#;
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid string"
    );
}

#[test]
fn test_valid_closure() {
    let engine = LintEngine::new(Config::default());
    let code = "let adder = {|x, y| $x + $y}";
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid closure"
    );
}

#[test]
fn test_valid_if_expression() {
    let engine = LintEngine::new(Config::default());
    let code = r#"
let x = 10
if $x > 5 {
    print "greater"
} else {
    print "lesser"
}
"#;
    let violations = engine.lint_source(code, None);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid if expression"
    );
}
