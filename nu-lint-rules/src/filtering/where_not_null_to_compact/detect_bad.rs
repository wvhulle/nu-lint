use super::RULE;

#[test]
fn detect_null_on_right() {
    RULE.assert_detects("[1 null] | where $it != null");
}

#[test]
fn detect_null_on_left() {
    RULE.assert_detects("[1 null] | where null != $it");
}
