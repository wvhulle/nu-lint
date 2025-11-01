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
fn ignores_multiline_function() {
    let code = r#"def process_data [
    input: string
    output: string
] {
    echo "processing"
}"#;
    rule().assert_ignores(code);
}
