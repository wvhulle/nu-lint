use super::RULE;

#[test]
fn replaces_simple_sort() {
    let source = "^sort file.txt";
    RULE.assert_fixed_contains(source, "sort");
}

#[test]
fn replaces_sort_in_pipeline() {
    let source = "ls | ^sort";
    RULE.assert_fixed_contains(source, "sort");
}

#[test]
fn ignores_builtin_sort() {
    let source = "ls | sort";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_sort_by() {
    let source = "ls | sort-by size";
    RULE.assert_ignores(source);
}
