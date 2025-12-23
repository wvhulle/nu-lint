use super::rule;

#[test]
fn ignores_short_single_line_list() {
    let code = "let items = [1 2 3]";
    rule().assert_ignores(code);
}

#[test]
fn ignores_multiline_list() {
    let code = r#"let items = [
    "first"
    "second"
    "third"
]"#;
    rule().assert_ignores(code);
}

#[test]
fn ignores_empty_list() {
    let code = "let items = []";
    rule().assert_ignores(code);
}

#[test]
fn ignores_short_list_with_few_items() {
    let code = r"let coords = [1.0, 2.0]";
    rule().assert_ignores(code);
}
