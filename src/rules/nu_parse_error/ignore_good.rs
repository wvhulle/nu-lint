use crate::{config::Config, engine::LintEngine, log::instrument};

#[test]
fn ignore_valid_let_statement() {
    let engine = LintEngine::new(Config::default());
    let code = "let x = 5";
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid let statement"
    );
}

#[test]
fn ignore_valid_function_definition() {
    let engine = LintEngine::new(Config::default());
    let code = r#"
def greet [name: string] {
    print $"Hello, ($name)!"
}
"#;
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid function"
    );
}

#[test]
fn ignore_valid_pipeline() {
    let engine = LintEngine::new(Config::default());
    let code = "ls | where size > 100 | sort-by name";
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid pipeline"
    );
}

#[test]
fn ignore_valid_list() {
    let engine = LintEngine::new(Config::default());
    let code = "let items = [1, 2, 3, 4, 5]";
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid list"
    );
}

#[test]
fn ignore_valid_record() {
    let engine = LintEngine::new(Config::default());
    let code = "let person = {name: 'John', age: 30}";
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid record"
    );
}

#[test]
fn ignore_valid_string() {
    let engine = LintEngine::new(Config::default());
    let code = r#"let greeting = "Hello, world!""#;
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid string"
    );
}

#[test]
fn ignore_valid_closure() {
    let engine = LintEngine::new(Config::default());
    let code = "let adder = {|x, y| $x + $y}";
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid closure"
    );
}

#[test]
fn ignore_valid_if_expression() {
    let engine = LintEngine::new(Config::default());
    let code = r#"
let x = 10
if $x > 5 {
    print "greater"
} else {
    print "lesser"
}
"#;
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid if expression"
    );
}

#[test]
fn ignore_complex_valid_use_statements() {
    instrument();
    let engine = LintEngine::new(Config::default());
    let code = r#"
use std/config light-theme
use std/config dark-theme

export def refresh-theme [] {
    let current_theme = "dark"
    match $current_theme {
        "dark" => {
            $env.config = ($env.config | merge {color_config: (dark-theme)})
        }
        "light" => {
            $env.config = ($env.config | merge {color_config: (light-theme)})
        }
        _ => {
            $env.config = ($env.config | merge {color_config: (dark-theme)})
        }
    }
}
"#;
    let violations = engine.lint_str(code);

    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id.as_ref() == "nu_parse_error"),
        "Expected no parse errors for valid use statements with std modules"
    );
}
