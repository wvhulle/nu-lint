use super::RULE;

#[test]
fn fix_filter_to_where() {
    RULE.assert_fixed_is("[1 2] | filter {|x| $x > 1}", "[1 2] | where {|x| $x > 1}");
}

#[test]
fn fix_filter_keeps_stored_closure() {
    RULE.assert_fixed_is(
        "let positive = {|x| $x > 0}; [1 -2] | filter $positive",
        "let positive = {|x| $x > 0}; [1 -2] | where $positive",
    );
}
