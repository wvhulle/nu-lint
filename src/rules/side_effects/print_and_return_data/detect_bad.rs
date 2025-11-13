use crate::log::instrument;

use super::rule;

#[test]
fn detects_function_with_print_and_return() {
    instrument();
    let bad_code = r#"
def fetch-data [] {
    print "Fetching..."
    http get https://api.example.com/data
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn detects_function_with_print_and_explicit_return() {
    instrument();
    let bad_code = r#"
def process-items [items: list] {
    print "Processing items"
    let result = ($items | where active)
    $result
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn detects_function_with_multiple_prints_and_return() {
    instrument();
    let bad_code = r#"
def complex-process [] {
    print "Step 1"
    let data = [1 2 3]
    print "Step 2"
    $data | each { |x| $x * 2 }
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn detects_exported_function_with_print_and_return() {
    instrument();
    let bad_code = r#"
export def get-config [] {
    print "Loading config..."
    open config.toml
}
"#;
    rule().assert_detects(bad_code);
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
    rule().assert_detects(bad_code);
}

#[test]
fn detects_function_with_nested_print() {
    instrument();
    let bad_code = r#"
def process-with-logging [] {
    let data = (do {
        print "Inner processing"
        [1 2 3]
    })
    $data | each { |x| $x * 2 }
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn detects_function_returning_list() {
    instrument();
    let bad_code = r#"
def list-items [] {
    print "Generating list"
    [1, 2, 3, 4, 5]
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn detects_function_returning_table() {
    instrument();
    let bad_code = r#"
def get-users [] {
    print "Fetching users"
    [[name age]; [Alice 30] [Bob 25]]
}
"#;
    rule().assert_detects(bad_code);
}
