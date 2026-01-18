use super::RULE;

#[test]
fn converts_reverse_flag() {
    RULE.assert_fixed_contains("^sort -r", "sort --reverse");
}

#[test]
fn converts_numeric_flag() {
    RULE.assert_fixed_contains("^sort -n", "sort --natural");
}

#[test]
fn converts_unique_flag() {
    RULE.assert_fixed_contains("^sort -u", "sort");
}

#[test]
fn converts_key_field() {
    RULE.assert_fixed_contains("^sort -k 2", "sort-by 2");
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
}

#[test]
fn combines_key_and_reverse() {
    RULE.assert_fixed_contains("^sort -k 3 -r", "sort-by 3 --reverse");
}
