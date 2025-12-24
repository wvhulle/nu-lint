use super::RULE;

#[test]
fn converts_tail_with_count_to_last() {
    let source = "^tail -10 file.txt";
    RULE.assert_replacement_contains(source, "last 10");
    RULE.assert_fix_explanation_contains(source, "last");
}
