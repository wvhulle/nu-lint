use crate::rules::replace_by_builtin::head::rule;

#[test]
fn replaces_head_with_first() {
    let source = "^head -5 file.txt";
    rule().assert_fix_contains(source, "first 5");
    rule().assert_fix_description_contains(source, "first");
}

#[test]
fn handles_head_without_count() {
    let source = "^head file.txt";
    rule().assert_fix_contains(source, "first 10");
}
