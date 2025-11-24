use super::rule;

#[test]
fn ignore_valid_let_statement() {
    let code = "let x = 5";
    rule().assert_ignores(code);
}

#[test]
fn ignore_valid_function_definition() {
    let code = r#"
def greet [name: string] {
    print $"Hello, ($name)!"
}
"#;
    rule().assert_ignores(code);
}

#[test]
fn ignore_valid_pipeline() {
    let code = "ls | where size > 100 | sort-by name";
    rule().assert_ignores(code);
}

#[test]
fn ignore_valid_list() {
    let code = "let items = [1, 2, 3, 4, 5]";
    rule().assert_ignores(code);
}

#[test]
fn ignore_valid_record() {
    let code = "let person = {name: 'John', age: 30}";
    rule().assert_ignores(code);
}

#[test]
fn ignore_valid_string() {
    let code = r#"let greeting = "Hello, world!""#;
    rule().assert_ignores(code);
}

#[test]
fn ignore_valid_closure() {
    let code = "let adder = {|x, y| $x + $y}";
    rule().assert_ignores(code);
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
    rule().assert_ignores(code);
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
    rule().assert_ignores(code);
}
