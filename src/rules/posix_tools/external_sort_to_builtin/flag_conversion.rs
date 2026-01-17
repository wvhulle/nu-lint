use super::RULE;

#[test]
fn converts_reverse_flag() {
    RULE.assert_fixed_contains("^sort -r", "sort --reverse");
    RULE.assert_fix_explanation_contains("^sort -r", "--reverse");
}

#[test]
fn converts_numeric_flag() {
    RULE.assert_fixed_contains("^sort -n", "sort --natural");
    RULE.assert_fix_explanation_contains("^sort -n", "natural");
    RULE.assert_fix_explanation_contains("^sort -n", "numeric");
}

#[test]
fn converts_unique_flag() {
    RULE.assert_fixed_contains("^sort -u", "sort");
    RULE.assert_fix_explanation_contains("^sort -u", "uniq");
    RULE.assert_fix_explanation_contains("^sort -u", "-u");
}

#[test]
fn converts_key_field() {
    RULE.assert_fixed_contains("^sort -k 2", "sort-by 2");
    RULE.assert_fix_explanation_contains("^sort -k 2", "sort-by");
    RULE.assert_fix_explanation_contains("^sort -k 2", "column");
}

#[test]
fn converts_key_field_compact_format() {
    RULE.assert_fixed_contains("^sort -k2", "sort-by 2");
}

#[test]
fn converts_ignore_case_flag() {
    RULE.assert_fixed_contains("^sort -f", "sort");
}

#[test]
fn combines_reverse_and_numeric() {
    RULE.assert_fixed_contains("^sort -nr", "sort --natural --reverse");
    RULE.assert_fix_explanation_contains("^sort -nr", "natural");
    RULE.assert_fix_explanation_contains("^sort -nr", "reverse");
}

#[test]
fn combines_key_and_reverse() {
    RULE.assert_fixed_contains("^sort -k 3 -r", "sort-by 3 --reverse");
}
