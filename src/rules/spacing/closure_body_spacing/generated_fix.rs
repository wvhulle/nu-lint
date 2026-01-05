use super::RULE;

#[test]
fn fix_closure_missing_both_spaces() {
    let bad = "[1 2] | each {|x|$x * 2}";
    let expected = "[1 2] | each {|x| $x * 2 }";
    RULE.assert_fixed_is(bad, expected);
}

#[test]
fn fix_empty_params_missing_space_after() {
    let bad = "[{||notify-long-command }]";
    let expected = "[{|| notify-long-command }]";
    RULE.assert_fixed_is(bad, expected);
}

#[test]
fn fix_closure_missing_space_before_closing() {
    let bad = "[1 2] | each {|x| $x * 2}";
    let expected = "[1 2] | each {|x| $x * 2 }";
    RULE.assert_fixed_is(bad, expected);
}
