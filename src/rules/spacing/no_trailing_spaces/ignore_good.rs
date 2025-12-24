use super::RULE;

#[test]
fn ignores_no_trailing_spaces() {
    let code = "let x = 42";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_internal_spaces() {
    let code = "let x = 42 + 24";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_empty_lines() {
    let code = "let x = 42\n\nlet y = 43";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_proper_indentation() {
    let code = "def test [] {\n    let x = 42\n}";
    RULE.assert_ignores(code);
}
