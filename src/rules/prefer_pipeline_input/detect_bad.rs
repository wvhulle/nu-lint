use super::rule;

#[test]
fn detect_single_data_parameter_with_each() {
    let bad_codes = vec![
        "def process-items [items] { $items | each { |x| $x * 2 } }",
        "def transform-data [data] { $data | each { |item| $item + 1 } }",
        "def double-values [values] { $values | each { |v| $v * 2 } }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_where() {
    let bad_codes = vec![
        "def filter-positive [numbers] { $numbers | where $it > 0 }",
        "def find-active [items] { $items | where active == true }",
        "def select-large [data] { $data | where size > 100 }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_select() {
    let bad_codes = vec![
        "def get-names [records] { $records | select name }",
        "def extract-info [data] { $data | select id name email }",
        "def pick-columns [table] { $table | select col1 col2 }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_sort_by() {
    let bad_codes = vec![
        "def sort-by-name [items] { $items | sort-by name }",
        "def order-data [records] { $records | sort-by modified }",
        "def arrange-list [data] { $data | sort-by size }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_group_by() {
    let bad_codes = vec![
        "def group-items [data] { $data | group-by category }",
        "def categorize [records] { $records | group-by type }",
        "def organize-data [items] { $items | group-by status }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_reduce() {
    let bad_codes = vec![
        "def sum-values [numbers] { $numbers | reduce { |acc, val| $acc + $val } }",
        "def calculate-total [data] { $data | reduce { |acc, item| $acc + $item.amount } }",
        "def aggregate [items] { $items | reduce { |acc, x| $acc + $x } }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_multiple_pipeline_operations() {
    let bad_codes = vec![
        "def process [data] { $data | where active | select name | sort-by name }",
        "def transform [items] { $items | each { |x| $x * 2 } | where $it > 5 }",
        "def analyze [records] { $records | group-by type | each { |group| $group | length } }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_math_operations() {
    let bad_codes = vec![
        "def sum-all [numbers] { $numbers | math sum }",
        "def get-average [values] { $values | math avg }",
        "def find-max [data] { $data | math max }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_single_data_parameter_with_length() {
    let bad_codes = vec![
        "def count-items [data] { $data | length }",
        "def get-size [items] { $items | length }",
        "def measure [collection] { $collection | length }",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
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
        rule().assert_detects(code);
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
        rule().assert_detects(code);
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
        rule().assert_detects(code);
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
        rule().assert_detects(code);
    }
}
