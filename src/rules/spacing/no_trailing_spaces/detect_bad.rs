use super::rule;

#[test]
fn detects_trailing_spaces() {
    let code = "let x = 42   ";
    rule().assert_count(code, 1);
}

#[test]
fn detects_trailing_tabs() {
    let code = "let x = 42\t\t";
    rule().assert_count(code, 1);
}

#[test]
fn detects_mixed_trailing_whitespace() {
    let code = "let x = 42 \t ";
    rule().assert_count(code, 1);
}

#[test]
fn detects_multiple_lines_with_trailing_spaces() {
    let code = "let x = 42  \nlet y = 43   ";
    rule().assert_count(code, 2);
}
