use super::RULE;

#[test]
fn ignore_set_builtins() {
    let good_code = r#"
let left = [3 1 2]
let right = [1 4]
$left | union $right
$left | intersect $right
$left | difference $right
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_where_in_without_uniq_keeps_duplicates() {
    let good_code = r#"
let allowed = [1 4]
[3 1 2 1 4] | where $it in $allowed
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_uniq_with_flags() {
    let good_code = r#"
[3 1 2] | append [1 4] | uniq --count
[3 1 2] | append [1 4] | uniq --repeated
['a' 'b'] | append ['A'] | uniq --ignore-case
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_append_scalar() {
    RULE.assert_ignores("[1 2] | append 3 | uniq");
}

#[test]
fn ignore_range_input() {
    RULE.assert_ignores("1..3 | append [5] | uniq");
}

#[test]
fn ignore_string_operand_is_substring_check() {
    RULE.assert_ignores("['a' 'z'] | where $it in 'abc' | uniq");
}

#[test]
fn ignore_untyped_operand() {
    let good_code = r#"
def keep [items: list<int>, allowed] {
    $items | where $it in $allowed | uniq
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_column_membership() {
    let good_code = r#"
let names = ['a']
[[name]; [a] [b]] | where name in $names | uniq
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_compound_row_condition() {
    let good_code = r#"
let allowed = [1 4]
[3 1 2 1 4] | where $it in $allowed and $it > 1 | uniq
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_pipeline_without_input() {
    RULE.assert_ignores("def f [] { append [1] | uniq }");
}
