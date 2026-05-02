use super::RULE;

#[test]
fn if_block_no_spaces() {
    let bad = "if true {echo 'yes'}";
    RULE.assert_detects(bad);
}

#[test]
fn if_block_missing_opening_space() {
    let bad = "if true {echo 'yes' }";
    RULE.assert_detects(bad);
}

#[test]
fn if_block_missing_closing_space() {
    let bad = "if true { echo 'yes'}";
    RULE.assert_detects(bad);
}
