use super::RULE;

#[test]
fn detects_violations_for_trailing_spaces() {
    let code = "let x = 42   ";
    RULE.assert_detects(code);
}
