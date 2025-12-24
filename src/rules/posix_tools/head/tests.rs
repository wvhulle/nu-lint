use super::RULE;

#[test]
fn converts_head_with_count_to_first() {
    let source = "^head -5 file.txt";
    RULE.assert_replacement_contains(source, "first 5");
    RULE.assert_fix_explanation_contains(source, "first");
}

#[test]
fn converts_head_without_count_to_first_ten() {
    let source = "^head file.txt";
    RULE.assert_replacement_contains(source, "first 10");
}
