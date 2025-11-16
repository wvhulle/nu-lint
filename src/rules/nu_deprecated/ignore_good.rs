use super::rule;

#[test]
fn ignore_simple_let_statement() {
    let code = "let x = 5";
    rule().assert_ignores(code);
}

#[test]
fn ignore_modern_function() {
    let code = r#"
def greet [name: string] {
    print $"Hello, ($name)!"
}
greet "World"
"#;
    rule().assert_ignores(code);
}
