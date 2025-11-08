use super::rule;

#[test]
fn ignores_short_function() {
    let code = "def add [x y] { $x + $y }";
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

#[test]
fn ignores_function_with_multiline_body() {
    let code = r"def transform [data] {
    $data
    | where column1 != null
    | select column1 column2
    | sort-by column1
}";
    rule().assert_ignores(code);
}
