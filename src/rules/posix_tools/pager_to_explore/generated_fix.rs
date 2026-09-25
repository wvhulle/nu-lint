use super::RULE;

#[test]
fn fix_less_to_explore() {
    RULE.assert_fixed_is("^less file.txt", "open --raw file.txt | explore");
}

#[test]
fn fix_more_to_explore() {
    RULE.assert_fixed_is(
        "^more documentation.txt",
        "open --raw documentation.txt | explore",
    );
}

#[test]
fn fix_follow_to_watch() {
    RULE.assert_fixed_is(
        "^less --follow log.txt",
        "watch log.txt { open --raw log.txt | lines | last 20 }",
    );
}

#[test]
fn fix_without_file_reads_pipeline_input() {
    RULE.assert_fixed_is("^less", "explore");
}

#[test]
fn no_fix_for_follow_without_file() {
    RULE.assert_detects_without_fix("^less -f");
}
