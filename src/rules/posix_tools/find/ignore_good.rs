use super::rule;

#[test]
fn ignores_builtin_find_for_data_filtering() {
    rule().assert_ignores(r"ls | find toml");
}

#[test]
fn ignores_builtin_find_with_regex() {
    rule().assert_ignores(r#"[abc bde arc abf] | find --regex "ab""#);
}

#[test]
fn ignores_builtin_find_on_strings() {
    rule().assert_ignores(r"'Cargo.toml' | find cargo");
}

#[test]
fn ignores_builtin_find_with_invert() {
    rule().assert_ignores(r"ls | find --invert test");
}

#[test]
fn ignores_builtin_find_with_columns() {
    rule().assert_ignores(r"ls | find pattern --columns [name]");
}
