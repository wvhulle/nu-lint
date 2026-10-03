use super::RULE;

#[test]
fn detect_external_read() {
    for code in [
        "^read name",
        "^read -rp 'Name: ' name",
        "^read",
        "^read -t 5 answer",
    ] {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_without_fix_when_no_variable_is_named() {
    RULE.assert_detects_without_fix("^read");
    RULE.assert_detects_without_fix("^read -t 5 answer");
    RULE.assert_detects_without_fix("$text | ^read line");
}
