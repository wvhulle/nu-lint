use super::rule;

#[test]
fn detects_long_single_line_list() {
    let code = r#"let items = ["very", "long", "list", "with", "many", "items", "that", "should", "be", "multiline"]"#;
    rule().assert_violation_count_exact(code, 1);
}

#[test]
fn detects_long_single_line_record() {
    let code = r#"let config = {name: "very long name", description: "very long description", version: "1.0.0"}"#;
    rule().assert_violation_count_exact(code, 1);
}

#[test]
fn detects_long_single_line_function() {
    let code = r#"def very_long_function_with_many_parameters [param1: string, param2: int, param3: bool] { echo "too long" }"#;
    rule().assert_violation_count_exact(code, 2);
}
