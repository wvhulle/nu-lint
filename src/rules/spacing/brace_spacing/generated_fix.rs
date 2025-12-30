use super::RULE;

#[test]
fn fix_closure_with_params_spacing() {
    let code = "let f = { |x| $x + 1 }";
    RULE.assert_fixed_contains(code, "{|x| $x + 1 }");
}

#[test]
fn fix_block_without_params_spacing() {
    let code = "if true {print 'hello'}";
    RULE.assert_fixed_contains(code, "{ print 'hello' }");
}

#[test]
fn fix_record_spacing() {
    let code = "{ key: value }";
    RULE.assert_fixed_is(code, "{key: value}");
}
