use super::RULE;
#[test]
fn detect_get() {
    let source = "{name: 1} | get -i name";
    RULE.assert_detects(source);
}
