use super::RULE;

#[test]
fn replaces_simple_uniq() {
    let source = "^uniq";
    RULE.assert_fixed_contains(source, "uniq");
}

#[test]
fn replaces_uniq_in_pipeline() {
    let source = "ls | ^uniq";
    RULE.assert_fixed_contains(source, "uniq");
}

#[test]
fn ignores_builtin_uniq() {
    let source = "ls | uniq";
    RULE.assert_ignores(source);
}

#[test]
fn ignores_builtin_uniq_by() {
    let source = "ls | uniq-by name";
    RULE.assert_ignores(source);
}
