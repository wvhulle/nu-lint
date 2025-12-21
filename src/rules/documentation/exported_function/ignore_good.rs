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

#[test]
fn exported_function_with_comment_and_example_attribute() {
    let source = r#"
# This function does something
@example "run the function" { my-command }
export def my-command [] {
    echo "hello"
}
"#;
    rule().assert_ignores(source);
}

#[test]
fn exported_function_with_comment_and_multiple_attributes() {
    let source = r#"
# Process data with multiple examples
@example "process a string" { process-data "test" }
@search-terms "data transform"
@category "processing"
export def process-data [input: string] {
    echo $input
}
"#;
    rule().assert_ignores(source);
}

#[test]
fn exported_function_with_comment_after_empty_lines() {
    let source = r#"

# A well-documented function
export def my-command [] {
    echo "hello"
}
"#;
    rule().assert_ignores(source);
}
