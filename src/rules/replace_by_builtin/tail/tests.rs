use crate::rules::replace_by_builtin::tail::rule;

#[test]
fn converts_tail_with_count_to_last() {
    let source = "^tail -10 file.txt";
    rule().assert_fix_contains(source, "last 10");
    rule().assert_fix_explanation_contains(source, "last");
}
