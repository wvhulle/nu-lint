use super::RULE;

#[test]
fn fix_record_with_leading_space() {
    let input = "{ x: 1, y: 2}";
    let expected = "{x: 1, y: 2}";
    RULE.assert_fixed_is(input, expected);
}

#[test]
fn fix_record_with_both_spaces() {
    let input = "{ x: 1, y: 2 }";
    let expected = "{x: 1, y: 2}";
    RULE.assert_fixed_is(input, expected);
}

#[test]
fn fix_record_with_trailing_space() {
    let input = "{x: 1, y: 2 }";
    let expected = "{x: 1, y: 2}";
    RULE.assert_fixed_is(input, expected);
}
