use super::rule;

#[test]
fn detects_long_single_line_function() {
    let code = r#"def very_long_function_with_many_parameters [param1: string, param2: int, param3: bool] { echo "too long" }"#;
    rule().assert_violation_count_exact(code, 1);
}

#[test]
fn detects_long_export_function() {
    let code = r"export def process_data_with_long_name [input: string, output: string, options: record] { $input | parse }";
    rule().assert_violation_count_exact(code, 1);
}

#[test]
fn detects_function_with_long_body() {
    let code = r"def transform [data] { $data | where column1 != null | select column1 column2 column3 | sort-by column1 }";
    rule().assert_violation_count_exact(code, 1);
}
