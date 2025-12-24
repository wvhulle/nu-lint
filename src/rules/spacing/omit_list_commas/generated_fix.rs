use super::RULE;

#[test]
fn detects_violations_for_comma_in_list() {
    let code = "let items = [1, 2, 3]";
    RULE.assert_detects(code);
}
