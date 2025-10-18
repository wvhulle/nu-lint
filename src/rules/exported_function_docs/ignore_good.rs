use super::rule;

#[test]
fn test_exported_function_with_docs() {
    let source = r#"
# This is a documented command
export def my-command [] {
    echo "hello"
}
"#;
    rule().assert_ignores(source);
}

#[test]
fn test_non_exported_function_without_docs() {
    let source = r#"
def my-command [] {
    echo "hello"
}
"#;
    rule().assert_ignores(source);
}

#[test]
fn test_exported_function_with_multi_line_docs() {
    let source = r"
# Process input data
# Returns the processed result
export def process-data [input: string] {
    echo $input
}
";
    rule().assert_ignores(source);
}
