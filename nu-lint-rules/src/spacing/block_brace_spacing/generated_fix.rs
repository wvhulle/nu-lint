use super::RULE;

#[test]
fn fix_if_block() {
    let input = "if true {echo 'yes'}";
    let expected = "if true { echo 'yes' }";
    RULE.assert_fixed_is(input, expected);
}

#[test]
fn fix_closure_without_params() {
    let input = "do {print 'hi'}";
    let expected = "do { print 'hi' }";
    RULE.assert_fixed_is(input, expected);
}
