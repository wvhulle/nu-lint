use super::RULE;

#[test]
fn fix_simple_each_operation() {
    let source = "def process-items [items] { $items | each { |x| $x * 2 } }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def process-items [] { each { |x| $x * 2 } }");
}

#[test]
fn fix_where_operation() {
    let source = "def filter-positive [numbers] { $numbers | where $it > 0 }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def filter-positive [] { where $it > 0 }");
}

#[test]
fn fix_select_operation() {
    let source = "def get-names [records] { $records | select name }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def get-names [] { select name }");
}

#[test]
fn fix_sort_by_operation() {
    let source = "def sort-by-name [items] { $items | sort-by name }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def sort-by-name [] { sort-by name }");
}

#[test]
fn fix_group_by_operation() {
    let source = "def group-items [data] { $data | group-by category }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def group-items [] { group-by category }");
}

#[test]
fn fix_reduce_operation() {
    let source = "def sum-values [numbers] { $numbers | reduce { |acc, val| $acc + $val } }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(
        source,
        "def sum-values [] { reduce { |acc, val| $acc + $val } }",
    );
}

#[test]
fn fix_multiple_pipeline_operations() {
    let source = "def process [data] { $data | where active | select name | sort-by name }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(
        source,
        "def process [] { where active | select name | sort-by name }",
    );
}

#[test]
fn fix_math_operations() {
    let source = "def sum-all [numbers] { $numbers | math sum }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def sum-all [] { math sum }");
}

#[test]
fn fix_length_operation() {
    let source = "def count-items [data] { $data | length }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def count-items [] { length }");
}

#[test]
fn fix_typed_list_parameter() {
    let source = "def process-list [items: list] { $items | each { |x| $x + 1 } }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def process-list [] { each { |x| $x + 1 } }");
}

#[test]
fn fix_typed_table_parameter() {
    let source = "def process-table [data: table] { $data | select name age }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def process-table [] { select name age }");
}

#[test]
fn fix_string_data_processing() {
    let source = "def split-lines [text] { $text | lines }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(source, "def split-lines [] { lines }");
}

#[test]
fn fix_multi_parameter_with_data_parameter() {
    let source = "def filter-range [data, min, max] { $data | where $it >= $min and $it <= $max }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(
        source,
        "def filter-range [min, max] { where $it >= $min and $it <= $max }",
    );
}

#[test]
fn fix_multi_parameter_with_config() {
    let source =
        "def process-with-config [items, config] { $items | each { |x| $x * $config.multiplier } }";

    RULE.assert_detects(source);
    RULE.assert_fixed_contains(
        source,
        "def process-with-config [config] { each { |x| $x * $config.multiplier } }",
    );
}
