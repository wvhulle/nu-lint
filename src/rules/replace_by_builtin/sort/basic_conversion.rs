use crate::rules::replace_by_builtin::sort::rule;

#[test]
fn replaces_simple_sort() {
    let source = "^sort file.txt";
    rule().assert_replacement_contains(source, "sort");
    rule().assert_fix_explanation_contains(source, "any data type");
}

#[test]
fn replaces_sort_in_pipeline() {
    let source = "ls | ^sort";
    rule().assert_replacement_contains(source, "sort");
}

#[test]
fn ignores_builtin_sort() {
    let source = "ls | sort";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_sort_by() {
    let source = "ls | sort-by size";
    rule().assert_ignores(source);
}
