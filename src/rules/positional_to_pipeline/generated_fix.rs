use super::rule;

#[test]
fn fix_simple_each_operation() {
    let source = "def process-items [items] { $items | each { |x| $x * 2 } }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def process-items [] { each { |x| $x * 2 } }");
}

#[test]
fn fix_where_operation() {
    let source = "def filter-positive [numbers] { $numbers | where $it > 0 }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def filter-positive [] { where $it > 0 }");
}

#[test]
fn fix_select_operation() {
    let source = "def get-names [records] { $records | select name }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def get-names [] { select name }");
}

#[test]
fn fix_sort_by_operation() {
    let source = "def sort-by-name [items] { $items | sort-by name }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def sort-by-name [] { sort-by name }");
}

#[test]
fn fix_group_by_operation() {
    let source = "def group-items [data] { $data | group-by category }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def group-items [] { group-by category }");
}

#[test]
fn fix_reduce_operation() {
    let source = "def sum-values [numbers] { $numbers | reduce { |acc, val| $acc + $val } }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(
        source,
        "def sum-values [] { reduce { |acc, val| $acc + $val } }",
    );
}

#[test]
fn fix_multiple_pipeline_operations() {
    let source = "def process [data] { $data | where active | select name | sort-by name }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(
        source,
        "def process [] { where active | select name | sort-by name }",
    );
}

#[test]
fn fix_math_operations() {
    let source = "def sum-all [numbers] { $numbers | math sum }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def sum-all [] { math sum }");
}

#[test]
fn fix_length_operation() {
    let source = "def count-items [data] { $data | length }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def count-items [] { length }");
}

#[test]
fn fix_typed_list_parameter() {
    let source = "def process-list [items: list] { $items | each { |x| $x + 1 } }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def process-list [] { each { |x| $x + 1 } }");
}

#[test]
fn fix_typed_table_parameter() {
    let source = "def process-table [data: table] { $data | select name age }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def process-table [] { select name age }");
}

#[test]
fn fix_string_data_processing() {
    let source = "def split-lines [text] { $text | lines }";

    rule().assert_detects(source);
    rule().assert_replacement_contains(source, "def split-lines [] { lines }");
}
