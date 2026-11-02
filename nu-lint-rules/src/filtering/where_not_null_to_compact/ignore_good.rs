use super::RULE;

#[test]
fn ignore_compact() {
    RULE.assert_ignores("[1 null] | compact");
}

#[test]
fn ignore_field_comparison() {
    RULE.assert_ignores("[{a: 1}] | where a != null");
}

#[test]
fn ignore_it_field_comparison() {
    RULE.assert_ignores("[{a: 1}] | where $it.a != null");
}

#[test]
fn ignore_equality_with_null() {
    RULE.assert_ignores("[1 null] | where $it == null");
}

#[test]
fn ignore_combined_condition() {
    RULE.assert_ignores("[1 null] | where $it != null and $it != 1");
}

#[test]
fn ignore_outer_variable_compared_inside_condition() {
    RULE.assert_ignores("let limit = 1\n[{a: 1}] | where $limit != null");
}
