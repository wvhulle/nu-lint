use super::RULE;

#[test]
fn converts_tail_with_count_to_last() {
    let source = "tail -10 file.txt";
    RULE.assert_replacement_contains(source, "open file.txt | lines | last 10");
}
