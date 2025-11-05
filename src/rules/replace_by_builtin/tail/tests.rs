use crate::rules::replace_by_builtin::tail::rule;

#[test]
fn replaces_tail_with_last() {
    let source = "^tail -10 file.txt";
    rule().assert_fix_contains(source, "last 10");
    rule().assert_fix_description_contains(source, "last");
}
