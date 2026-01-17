use super::RULE;

#[test]
fn exported_function_missing_documentation() {
    let source = r#"
export def my-command [] {
    echo "hello"
}
"#;
    RULE.assert_count(source, 1);
}

#[test]
fn exported_function_with_params_missing_documentation() {
    let source = r"
export def process-data [input: string, output: string] {
    echo $input | save $output
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn exported_function_with_only_example_attribute() {
    let source = r#"
@example "run the function" { my-command }
export def my-command [] {
    echo "hello"
}
"#;
    RULE.assert_count(source, 1);
}

#[test]
fn exported_function_with_multiple_attributes_but_no_comment() {
    let source = r#"
@example "process a string" { process-data "test" }
@search-terms "data transform"
export def process-data [input: string] {
    echo $input
}
"#;
    RULE.assert_count(source, 1);
}

#[test]
fn exported_function_with_shebang_comment() {
    let source = r#"
## This is not a documentation comment
export def my-command [] {
    echo "hello"
}
"#;
    RULE.assert_count(source, 1);
}
