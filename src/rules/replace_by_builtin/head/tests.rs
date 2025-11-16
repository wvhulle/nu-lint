use crate::rules::replace_by_builtin::head::rule;

#[test]
fn converts_head_with_count_to_first() {
    let source = "^head -5 file.txt";
    rule().assert_fix_contains(source, "first 5");
    rule().assert_fix_explanation_contains(source, "first");
}

#[test]
fn converts_head_without_count_to_first_ten() {
    let source = "^head file.txt";
    rule().assert_fix_contains(source, "first 10");
}
