use super::RULE;

#[test]
fn detect_get_short_flag() {
    RULE.assert_detects("{name: 1} | get -i name");
}

#[test]
fn detect_select_short_flag() {
    RULE.assert_detects("[[a]; [1]] | select -i a");
}

#[test]
fn detect_reject_long_flag() {
    RULE.assert_detects("[[a]; [1]] | reject --ignore-errors a");
}
