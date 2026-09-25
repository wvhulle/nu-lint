use super::RULE;

#[test]
fn fix_bsd_style_count_to_last() {
    RULE.assert_fixed_is(
        "^tail -10 file.txt",
        "open --raw file.txt | lines | last 10",
    );
}

#[test]
fn fix_short_lines_flag_with_separate_value() {
    RULE.assert_fixed_is(
        "^tail -n 5 notes.txt",
        "open --raw notes.txt | lines | last 5",
    );
}

#[test]
fn fix_short_lines_flag_with_attached_value() {
    RULE.assert_fixed_is(
        "^tail -n5 notes.txt",
        "open --raw notes.txt | lines | last 5",
    );
}

#[test]
fn fix_long_lines_flag_with_separate_value() {
    RULE.assert_fixed_is(
        "^tail --lines 3 notes.txt",
        "open --raw notes.txt | lines | last 3",
    );
}

#[test]
fn fix_long_lines_flag_with_attached_value() {
    RULE.assert_fixed_is(
        "^tail --lines=3 notes.txt",
        "open --raw notes.txt | lines | last 3",
    );
}

#[test]
fn fix_without_count_uses_tail_default() {
    RULE.assert_fixed_is("^tail notes.txt", "open --raw notes.txt | lines | last 10");
}

#[test]
fn fix_without_file_reads_pipeline_input() {
    RULE.assert_fixed_is("^tail -n 5", "last 5");
}

#[test]
fn fix_follow_to_watch() {
    RULE.assert_fixed_is(
        "^tail -f log.txt",
        "watch log.txt { open --raw log.txt | lines | last 20 }",
    );
}

#[test]
fn fix_follow_keeps_requested_count() {
    RULE.assert_fixed_is(
        "^tail -f -n 3 log.txt",
        "watch log.txt { open --raw log.txt | lines | last 3 }",
    );
}

#[test]
fn no_fix_for_follow_without_file() {
    RULE.assert_detects_without_fix("^tail -f");
}

#[test]
fn no_fix_for_offset_count() {
    RULE.assert_detects_without_fix("^tail -n +5 notes.txt");
}

#[test]
fn no_fix_when_count_value_is_missing() {
    RULE.assert_detects_without_fix("^tail -n");
}
