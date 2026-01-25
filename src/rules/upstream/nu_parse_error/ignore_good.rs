use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_config_nu() {
    init_test_log();
    let code = "config nu --doc | nu-highlight | to html --html-color";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_let_statement() {
    let code = "let x = 5";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_function_definition() {
    let code = r#"
def greet [name: string] {
    print $"Hello, ($name)!"
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_pipeline() {
    let code = "ls | where size > 100 | sort-by name";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_list() {
    let code = "let items = [1, 2, 3, 4, 5]";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_record() {
    let code = "let person = {name: 'John', age: 30}";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_string() {
    let code = r#"let greeting = "Hello, world!""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_closure() {
    let code = "let adder = {|x, y| $x + $y}";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_path_self() {
    let code = "const SELF = path self .";
    RULE.assert_ignores(code);
}

#[test]
fn ignore_valid_if_expression() {
    let code = r#"
let x = 10
if $x > 5 {
    print "greater"
} else {
    print "lesser"
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_complex_valid_use_statements() {
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
    RULE.assert_ignores(code);
}
