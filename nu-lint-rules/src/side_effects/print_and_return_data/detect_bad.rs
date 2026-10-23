use nu_lint_analysis::log::init_test_log;

use super::RULE;

#[test]
fn detects_function_with_print_and_return() {
    init_test_log();
    let bad_code = r#"
def fetch-data [] {
    print "Fetching..."
    http get https://api.example.com/data
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_function_with_print_in_conditional() {
    init_test_log();
    let bad_code = r#"
def conditional-fetch [verbose: bool] {
    if $verbose {
        print "Fetching data..."
    }
    http get https://api.example.com/data
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_function_returning_data_structure() {
    init_test_log();
    let bad_code = r#"
def list-items [] {
    print "Generating list"
    [1, 2, 3, 4, 5]
}
"#;
    RULE.assert_detects(bad_code);
}
