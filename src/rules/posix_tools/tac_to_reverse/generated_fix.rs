use super::RULE;

#[test]
fn fix_simple_tac() {
    let source = "^tac file.txt";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open --raw file.txt | lines | reverse");
}

#[test]
fn fix_tac_log_file() {
    let source = "^tac file.log";
    RULE.assert_fixed_contains(source, "open --raw file.log | lines | reverse");
}

#[test]
fn fix_preserves_filename() {
    let source = "^tac my-log-file.log";
    RULE.assert_fixed_contains(source, "my-log-file.log");
}
