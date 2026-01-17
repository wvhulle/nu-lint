use super::RULE;

#[test]
fn fix_tail_with_count_to_last() {
    let source = "tail -10 file.txt";
    RULE.assert_fixed_contains(source, "open file.txt | lines | last 10");
}

#[test]
fn fix_tail_follow_to_watch() {
    let source = "tail -f log.txt";
    RULE.assert_fixed_contains(source, "watch log.txt");
}

#[test]
fn fix_tail_follow_long_to_watch() {
    let source = "tail --follow log.txt";
    RULE.assert_fixed_contains(source, "watch log.txt");
}

#[test]
fn fix_tail_follow_uppercase_to_watch() {
    let source = "tail -F log.txt";
    RULE.assert_fixed_contains(source, "watch log.txt");
}

#[test]
fn fix_tail_follow_explanation_mentions_watch() {
    let source = "tail -f log.txt";
    RULE.assert_fix_explanation_contains(source, "watch");
}

#[test]
fn fix_tail_follow_explanation_mentions_tail_f() {
    let source = "tail -f log.txt";
    RULE.assert_fix_explanation_contains(source, "tail -f");
}

#[test]
fn fix_tail_count_explanation_mentions_last() {
    let source = "tail -10 file.txt";
    RULE.assert_fix_explanation_contains(source, "last");
}

#[test]
fn fix_preserves_filename() {
    let source = "tail -5 my-log-file.log";
    RULE.assert_fixed_contains(source, "my-log-file.log");
}
