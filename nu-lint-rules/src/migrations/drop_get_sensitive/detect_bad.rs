use super::RULE;

#[test]
fn detect_get_sensitive() {
    RULE.assert_detects("{a: 1} | get --sensitive a");
}

#[test]
fn detect_get_sensitive_between_cell_paths() {
    RULE.assert_detects("{a: {b: 1}} | get a --sensitive b");
}
