use super::RULE;

#[test]
fn ignores_list_without_commas() {
    let code = "let items = [1 2 3]";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_empty_list() {
    let code = "let empty = []";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_single_item_list() {
    let code = "let single = [42]";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_multiline_list_without_commas() {
    let code = r#"let items = [
    "first"
    "second"
    "third"
]"#;
    RULE.assert_ignores(code);
}
