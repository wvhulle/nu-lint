use super::rule;

#[test]
fn detect_redundant_in_at_pipeline_start() {
    let _ = env_logger::try_init();

    let bad_codes = vec![
        // Commands that start with redundant $in
        "def get-field [field] { $in | get $field }",
        "def has-key [key] { $in | columns | any { |col| $col == $key } }",
        "def select-column [column] { $in | select $column }",
        "def sort-by-field [field] { $in | sort-by $field }",
        "def multiply [factor] { $in | each { |x| $x * $factor } }",
        "def take-first [n] { $in | first $n }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_redundant_in_no_parameters() {
    let bad_codes = vec![
        // Commands with no parameters that use redundant $in
        "def process [] { $in | where active }",
        "def transform [] { $in | each { |x| $x * 2 } }",
        "def filter-positive [] { $in | where $it > 0 }",
        "def get-names [] { $in | select name }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_redundant_in_with_spaces() {
    let bad_codes = vec![
        // Test different spacing patterns
        "def process [] { $in| where active }", // No space after $in
        "def process [] {    $in | where active }", // Leading spaces
        "def process [] {\n    $in | where active\n}", // Multi-line
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_redundant_in_complex_pipelines() {
    let bad_codes = vec![
        // Complex pipelines that start with redundant $in
        "def process [] { $in | where active | select name | sort-by name }",
        "def calculate [] { $in | each { |x| $x * 2 } | math sum }",
        "def filter-and-count [] { $in | where $it > 5 | length }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}
