use super::RULE;

#[test]
fn fix_list_filter() {
    RULE.assert_fixed_is("[1 null] | where $it != null", "[1 null] | compact");
}

#[test]
fn fix_keeps_following_commands() {
    RULE.assert_fixed_is(
        "[1 null] | where null != $it | length",
        "[1 null] | compact | length",
    );
}
