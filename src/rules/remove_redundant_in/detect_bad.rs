use super::rule;
use crate::log::instrument;

#[test]
fn detect_redundant_in_at_pipeline_start() {
    instrument();

    let bad_codes = vec![
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
        "def process [] { $in| where active }",
        "def process [] {    $in | where active }",
        "def process [] {\n    $in | where active\n}",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_redundant_in_complex_pipelines() {
    let bad_codes = vec![
        "def process [] { $in | where active | select name | sort-by name }",
        "def calculate [] { $in | each { |x| $x * 2 } | math sum }",
        "def filter-and-count [] { $in | where $it > 5 | length }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}
