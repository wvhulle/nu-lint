use super::rule;

#[test]
fn exported_function_with_documentation() {
    let source = r#"
# This is a documented command
export def my-command [] {
    echo "hello"
}
"#;
    rule().assert_ignores(source);
}

#[test]
fn non_exported_function_ignores_documentation_requirement() {
    let source = r#"
def my-command [] {
    echo "hello"
}
"#;
    rule().assert_ignores(source);
}

#[test]
fn exported_function_with_multiline_documentation() {
    let source = r"
# Process input data
# Returns the processed result
export def process-data [input: string] {
    echo $input
}
";
    rule().assert_ignores(source);
}
