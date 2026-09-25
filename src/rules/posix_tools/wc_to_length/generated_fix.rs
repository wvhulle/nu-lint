use super::RULE;

#[test]
fn fix_line_count_of_pipeline_input() {
    RULE.assert_fixed_is("^wc -l", "lines | length");
}

#[test]
fn fix_line_count_keeps_file_operand() {
    RULE.assert_fixed_is("^wc -l notes.txt", "open --raw notes.txt | lines | length");
}

#[test]
fn fix_item_count_keeps_file_operand() {
    RULE.assert_fixed_is("^wc notes.txt", "open --raw notes.txt | length");
}

#[test]
fn fix_item_count_of_pipeline_input() {
    RULE.assert_fixed_is("^wc", "length");
}

#[test]
fn no_fix_for_multiple_files() {
    RULE.assert_detects_without_fix("^wc -l a.txt b.txt");
}
