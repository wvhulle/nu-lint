use super::rule;

#[test]
fn detects_violations_for_trailing_spaces() {
    let code = "let x = 42   ";
    rule().assert_detects(code);
}
