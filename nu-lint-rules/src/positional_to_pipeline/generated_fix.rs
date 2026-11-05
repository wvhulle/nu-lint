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

#[test]
fn fix_replaces_whole_parameter_list_with_brackets_in_comment() {
    RULE.assert_fixed_is(
        "def get-first [\n    items # see [docs]\n] { $items | first }",
        "def get-first [] { first }",
    );
}

#[test]
fn no_fix_when_usage_is_not_a_pipeline_head() {
    RULE.assert_detects_without_fix(
        "def pick [items] { ls | where name in $items | first; $items | first }",
    );
}

#[test]
fn no_fix_when_usage_is_inside_a_closure() {
    RULE.assert_detects_without_fix(
        "def scale [items] { $items | each { |value| $value * ($items | length) } }",
    );
}

#[test]
fn fix_removes_head_without_spaces_around_pipe() {
    RULE.assert_fixed_is(
        "def get-first [items] { $items|first }",
        "def get-first [] { first }",
    );
}

#[test]
fn fix_keeps_types_of_other_parameters() {
    RULE.assert_fixed_is(
        "def tag [items: list<string>, label: string] { $items | each { |item| $label } }",
        "def tag [label: string] { each { |item| $label } }",
    );
}

#[test]
fn fix_keeps_optional_parameter() {
    RULE.assert_fixed_is(
        "def pick [items, count?: int] { $items | first $count }",
        "def pick [count?: int] { first $count }",
    );
}

#[test]
fn fix_keeps_flag() {
    RULE.assert_fixed_is(
        "def pick [items, --reverse] { $items | first }",
        "def pick [--reverse] { first }",
    );
}

#[test]
fn fix_keeps_rest_parameter() {
    RULE.assert_fixed_is(
        "def pick [items, ...others] { $items | append $others }",
        "def pick [...others] { append $others }",
    );
}

#[test]
fn fix_removes_last_parameter_without_trailing_comma() {
    RULE.assert_fixed_is(
        "def pick [--reverse, items] { $items | first }",
        "def pick [--reverse] { first }",
    );
}

#[test]
fn no_fix_when_signature_declares_input_type() {
    RULE.assert_detects_without_fix(
        "def repo-name [path: string]: nothing -> string { $path | path basename }",
    );
}

#[test]
fn no_fix_when_command_is_called_in_file() {
    RULE.assert_detects_without_fix(
        "def pick [items] { $items | first }\ndef main [] { pick [1 2] }",
    );
}

#[test]
fn no_fix_when_signature_declares_any_types() {
    RULE.assert_detects_without_fix(
        "def repo-name [path: string]: any -> any { $path | path basename }",
    );
}
