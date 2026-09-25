use super::RULE;

#[test]
fn fix_tac_with_file() {
    RULE.assert_fixed_is("^tac file.txt", "open --raw file.txt | lines | reverse");
}

#[test]
fn fix_without_file_reads_pipeline_input() {
    RULE.assert_fixed_is("^tac", "lines | reverse");
}
