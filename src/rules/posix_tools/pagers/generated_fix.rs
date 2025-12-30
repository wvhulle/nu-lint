use super::RULE;

#[test]
fn fix_less_to_explore() {
    let source = "^less file.txt";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "open --raw file.txt | explore");
}

#[test]
fn fix_more_to_explore() {
    let source = "^more documentation.txt";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "open --raw documentation.txt | explore");
}

#[test]
fn fix_less_follow_to_watch() {
    let source = "^less -f log.txt";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "watch log.txt");
}

#[test]
fn fix_less_follow_long_to_watch() {
    let source = "^less --follow log.txt";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "watch log.txt");
}

#[test]
fn fix_explanation_mentions_explore() {
    let source = "^less file.txt";
    RULE.assert_fix_explanation_contains(source, "explore");
}

#[test]
fn fix_explanation_mentions_structured() {
    let source = "^less file.txt";
    RULE.assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_follow_explanation_mentions_watch() {
    let source = "^less -f log.txt";
    RULE.assert_fix_explanation_contains(source, "watch");
}

#[test]
fn fix_follow_explanation_mentions_tail_f() {
    let source = "^less -f log.txt";
    RULE.assert_fix_explanation_contains(source, "tail -f");
}

#[test]
fn fix_preserves_filename() {
    let source = "^less my-complex-log.log";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "my-complex-log.log");
}
