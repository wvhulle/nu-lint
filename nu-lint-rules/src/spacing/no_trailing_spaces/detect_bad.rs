use super::RULE;

#[test]
fn detects_trailing_spaces() {
    let code = "let x = 42   ";
    RULE.assert_count(code, 1);
}

#[test]
fn detects_trailing_tabs() {
    let code = "let x = 42\t\t";
    RULE.assert_count(code, 1);
}

#[test]
fn detects_mixed_trailing_whitespace() {
    let code = "let x = 42 \t ";
    RULE.assert_count(code, 1);
}

#[test]
fn detects_multiple_lines_with_trailing_spaces() {
    let code = "let x = 42  \nlet y = 43   ";
    RULE.assert_count(code, 2);
}
