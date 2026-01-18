use super::RULE;

#[test]
fn fix_less_to_explore() {
    let source = "^less file.txt";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open --raw file.txt | explore");
}

#[test]
fn fix_more_to_explore() {
    let source = "^more documentation.txt";
    RULE.assert_fixed_contains(source, "open --raw documentation.txt | explore");
}

#[test]
fn fix_less_follow_to_watch() {
    let source = "^less -f log.txt";
    RULE.assert_fixed_contains(source, "watch log.txt");
}

#[test]
fn fix_less_follow_long_to_watch() {
    let source = "^less --follow log.txt";
    RULE.assert_fixed_contains(source, "watch log.txt");
}

#[test]
fn fix_preserves_filename() {
    let source = "^less my-complex-log.log";
    RULE.assert_fixed_contains(source, "my-complex-log.log");
}
