use super::RULE;

#[test]
fn detect_current_date() {
    for code in [
        "^date",
        "^date -u",
        "^date +%Y-%m-%d",
        r#"^date "+%F %T""#,
        "^date --utc +%s",
        "let stamp = (^date +%-d)",
    ] {
        RULE.assert_detects(code);
    }
}
