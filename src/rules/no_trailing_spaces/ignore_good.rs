use super::rule;

#[test]
fn ignores_no_trailing_spaces() {
    let code = "let x = 42";
    rule().assert_ignores(code);
}

#[test]
fn ignores_internal_spaces() {
    let code = "let x = 42 + 24";
    rule().assert_ignores(code);
}

#[test]
fn ignores_empty_lines() {
    let code = "let x = 42\n\nlet y = 43";
    rule().assert_ignores(code);
}

#[test]
fn ignores_proper_indentation() {
    let code = "def test [] {\n    let x = 42\n}";
    rule().assert_ignores(code);
}
