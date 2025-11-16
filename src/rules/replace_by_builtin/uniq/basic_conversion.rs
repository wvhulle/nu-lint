use crate::rules::replace_by_builtin::uniq::rule;

#[test]
fn replaces_simple_uniq() {
    let source = "^uniq";
    rule().assert_fix_contains(source, "uniq");
    rule().assert_fix_explanation_contains(source, "structured data");
}

#[test]
fn replaces_uniq_in_pipeline() {
    let source = "ls | ^uniq";
    rule().assert_fix_contains(source, "uniq");
}

#[test]
fn ignores_builtin_uniq() {
    let source = "ls | uniq";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_uniq_by() {
    let source = "ls | uniq-by name";
    rule().assert_ignores(source);
}
