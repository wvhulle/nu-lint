use super::rule;

#[test]
fn exported_function_missing_documentation() {
    let source = r#"
export def my-command [] {
    echo "hello"
}
"#;
    rule().assert_violation_count_exact(source, 1);
}

#[test]
fn exported_function_with_params_missing_documentation() {
    let source = r"
export def process-data [input: string, output: string] {
    echo $input | save $output
}
";
    rule().assert_violation_count_exact(source, 1);
}
