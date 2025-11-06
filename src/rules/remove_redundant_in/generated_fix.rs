use super::rule;

#[test]
fn fix_get_field_operation() {
    let source = "def get-field [field] { $in | get $field }";

    rule().assert_detects(source);
    rule().assert_fix_contains(source, "def get-field [field] { get $field }");
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}

#[test]
fn fix_select_operation() {
    let source = "def select-column [column] { $in | select $column }";

    rule().assert_detects(source);
    rule().assert_fix_contains(source, "def select-column [column] { select $column }");
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}

#[test]
fn fix_each_operation() {
    let source = "def multiply [factor] { $in | each { |x| $x * $factor } }";

    rule().assert_detects(source);
    rule().assert_fix_contains(
        source,
        "def multiply [factor] { each { |x| $x * $factor } }",
    );
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}

#[test]
fn fix_no_parameters() {
    let source = "def process [] { $in | where active }";

    rule().assert_detects(source);
    rule().assert_fix_contains(source, "def process [] { where active }");
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}

#[test]
fn fix_complex_pipeline() {
    let source = "def process [] { $in | where active | select name | sort-by name }";

    rule().assert_detects(source);
    rule().assert_fix_contains(
        source,
        "def process [] { where active | select name | sort-by name }",
    );
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}

#[test]
fn fix_no_space_after_in() {
    let source = "def filter [] { $in| where $it > 0 }";

    rule().assert_detects(source);
    rule().assert_fix_contains(source, "def filter [] { where $it > 0 }");
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}

#[test]
fn fix_sort_by_operation() {
    let source = "def sort-by-field [field] { $in | sort-by $field }";

    rule().assert_detects(source);
    rule().assert_fix_contains(source, "def sort-by-field [field] { sort-by $field }");
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}

#[test]
fn fix_first_operation() {
    let source = "def take-first [n] { $in | first $n }";

    rule().assert_detects(source);
    rule().assert_fix_contains(source, "def take-first [n] { first $n }");
    rule().assert_suggestion_contains(source, "Remove redundant $in");
}
