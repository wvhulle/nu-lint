use super::RULE;

#[test]
fn test_block_without_spaces() {
    let bad = "do {print 'test'}";
    RULE.assert_detects(bad);
}

#[test]
fn test_block_missing_opening_space() {
    let bad = "do {print 'test' }";
    RULE.assert_detects(bad);
}

#[test]
fn test_block_missing_closing_space() {
    let bad = "do { print 'test'}";
    RULE.assert_detects(bad);
}

#[test]
fn test_if_block_no_spaces() {
    let bad = "if true {echo 'yes'}";
    RULE.assert_detects(bad);
}

#[test]
fn test_closure_no_params_no_spaces() {
    // Closure without explicit params (like { echo foo }) still needs spaces
    let bad = "[1 2] | each {$in * 2}";
    RULE.assert_detects(bad);
}
