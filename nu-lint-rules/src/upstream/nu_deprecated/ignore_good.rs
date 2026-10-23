use super::RULE;

#[test]
fn ignore_simple_let_statement() {
    RULE.assert_ignores("let x = 5");
}

#[test]
fn ignore_modern_function() {
    RULE.assert_ignores(
        r#"
def greet [name: string] {
    print $"Hello, ($name)!"
}
greet "World"
"#,
    );
}
