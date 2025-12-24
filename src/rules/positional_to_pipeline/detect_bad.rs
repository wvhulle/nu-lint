use super::RULE;

#[test]
fn detect_single_data_parameter_with_pipeline_operations() {
    let test_cases = vec![
        // each operations
        "def process-items [items] { $items | each { |x| $x * 2 } }",
        "def transform-data [data] { $data | each { |item| $item + 1 } }",
        // where operations
        "def filter-positive [numbers] { $numbers | where $it > 0 }",
        "def find-active [items] { $items | where active == true }",
        // select operations
        "def get-names [records] { $records | select name }",
        "def extract-info [data] { $data | select id name email }",
        // sort-by operations
        "def sort-by-name [items] { $items | sort-by name }",
        "def order-data [records] { $records | sort-by modified }",
        // group-by operations
        "def group-items [data] { $data | group-by category }",
        "def categorize [records] { $records | group-by type }",
        // reduce operations
        "def sum-values [numbers] { $numbers | reduce { |acc, val| $acc + $val } }",
        "def calculate-total [data] { $data | reduce { |acc, item| $acc + $item.amount } }",
        // multiple pipeline operations
        "def process [data] { $data | where active | select name | sort-by name }",
        "def transform [items] { $items | each { |x| $x * 2 } | where $it > 5 }",
        // math operations
        "def sum-all [numbers] { $numbers | math sum }",
        "def get-average [values] { $values | math avg }",
        // length operations
        "def count-items [data] { $data | length }",
        "def get-size [items] { $items | length }",
    ];

    for code in test_cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_list_typed_single_parameter() {
    let bad_codes = vec![
        "def process-list [items: list] { $items | each { |x| $x + 1 } }",
        "def filter-list [data: list<int>] { $data | where $it > 0 }",
        "def sort-list [values: list<string>] { $values | sort }",
    ];

    for code in bad_codes {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_table_typed_single_parameter() {
    let bad_codes = vec![
        "def process-table [data: table] { $data | select name age }",
        "def filter-table [records: table] { $records | where active }",
        "def sort-table [data: table] { $data | sort-by name }",
    ];

    for code in bad_codes {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_record_typed_single_parameter() {
    let bad_codes = vec![
        "def process-record [data: record] { $data | select name }",
        "def extract-fields [item: record] { $item | get name email }",
        "def transform-record [record: record] { $record | upsert processed true }",
    ];

    for code in bad_codes {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_string_parameter_used_for_data_processing() {
    let bad_codes = vec![
        "def split-lines [text] { $text | lines }",
        "def parse-json [json_str] { $json_str | from json }",
        "def clean-text [content] { $content | str trim | str replace 'old' 'new' }",
    ];

    for code in bad_codes {
        RULE.assert_detects(code);
    }
}
