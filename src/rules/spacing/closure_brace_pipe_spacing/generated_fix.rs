use super::RULE;

#[test]
fn fix_closure_param_spacing() {
    let input = "[[status]; [UP]] | all { |el| $el.status == UP }";
    let expected = "[[status]; [UP]] | all {|el| $el.status == UP }";
    RULE.assert_fixed_is(input, expected);
}

#[test]
fn fix_reduce_closure() {
    let input = "[1 2 3 4] | reduce { |elt, acc| $elt + $acc }";
    let expected = "[1 2 3 4] | reduce {|elt, acc| $elt + $acc }";
    RULE.assert_fixed_is(input, expected);
}
