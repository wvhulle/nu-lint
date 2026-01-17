use super::RULE;

#[test]
fn fix_block_without_spaces() {
    let input = "do {print 'test'}";
    let expected = "do { print 'test' }";
    RULE.assert_fixed_is(input, expected);
}

#[test]
fn fix_if_block() {
    let input = "if true {echo 'yes'}";
    let expected = "if true { echo 'yes' }";
    RULE.assert_fixed_is(input, expected);
}

#[test]
fn fix_closure_no_params() {
    let input = "[1 2] | each {$in * 2}";
    let expected = "[1 2] | each { $in * 2 }";
    RULE.assert_fixed_is(input, expected);
}
