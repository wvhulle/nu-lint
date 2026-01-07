use super::RULE;

#[test]
fn converts_head_with_count_to_first() {
    let source = "^head -n 10 file.txt";
    RULE.assert_fixed_is(source, "open file.txt | lines | first 10");
}
