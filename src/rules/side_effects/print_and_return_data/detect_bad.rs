use super::RULE;
use crate::log::instrument;

#[test]
fn detects_function_with_print_and_return() {
    instrument();
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
    instrument();
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
    instrument();
    let bad_code = r#"
def list-items [] {
    print "Generating list"
    [1, 2, 3, 4, 5]
}
"#;
    RULE.assert_detects(bad_code);
}
