use super::RULE;

#[test]
fn ignore_where_closure() {
    RULE.assert_ignores("[1 2] | where {|x| $x > 1}");
}

#[test]
fn ignore_where_row_condition() {
    RULE.assert_ignores("[[a]; [1]] | where a > 0");
}

#[test]
fn ignore_custom_command_named_filter() {
    RULE.assert_ignores("def filter [] { 1 }; filter");
}
