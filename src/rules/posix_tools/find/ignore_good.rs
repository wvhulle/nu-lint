use super::RULE;

#[test]
fn ignores_builtin_find_for_data_filtering() {
    RULE.assert_ignores(r"ls | find toml");
}

#[test]
fn ignores_builtin_find_with_regex() {
    RULE.assert_ignores(r#"[abc bde arc abf] | find --regex "ab""#);
}

#[test]
fn ignores_builtin_find_on_strings() {
    RULE.assert_ignores(r"'Cargo.toml' | find cargo");
}

#[test]
fn ignores_builtin_find_with_invert() {
    RULE.assert_ignores(r"ls | find --invert test");
}

#[test]
fn ignores_builtin_find_with_columns() {
    RULE.assert_ignores(r"ls | find pattern --columns [name]");
}
