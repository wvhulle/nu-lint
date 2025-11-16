use crate::rules::replace_by_builtin::sort::rule;

#[test]
fn converts_reverse_flag() {
    rule().assert_fix_contains("^sort -r", "sort --reverse");
    rule().assert_fix_explanation_contains("^sort -r", "--reverse");
}

#[test]
fn converts_numeric_flag() {
    rule().assert_fix_contains("^sort -n", "sort --natural");
    rule().assert_fix_explanation_contains("^sort -n", "natural");
    rule().assert_fix_explanation_contains("^sort -n", "numeric");
}

#[test]
fn converts_unique_flag() {
    rule().assert_fix_contains("^sort -u", "sort");
    rule().assert_fix_explanation_contains("^sort -u", "uniq");
    rule().assert_fix_explanation_contains("^sort -u", "-u");
}

#[test]
fn converts_key_field() {
    rule().assert_fix_contains("^sort -k 2", "sort-by 2");
    rule().assert_fix_explanation_contains("^sort -k 2", "sort-by");
    rule().assert_fix_explanation_contains("^sort -k 2", "column");
}

#[test]
fn converts_key_field_compact_format() {
    rule().assert_fix_contains("^sort -k2", "sort-by 2");
}

#[test]
fn converts_ignore_case_flag() {
    rule().assert_fix_contains("^sort -f", "sort");
}

#[test]
fn combines_reverse_and_numeric() {
    rule().assert_fix_contains("^sort -nr", "sort --natural --reverse");
    rule().assert_fix_explanation_contains("^sort -nr", "natural");
    rule().assert_fix_explanation_contains("^sort -nr", "reverse");
}

#[test]
fn combines_key_and_reverse() {
    rule().assert_fix_contains("^sort -k 3 -r", "sort-by 3 --reverse");
}
