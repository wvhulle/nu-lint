use super::RULE;

#[test]
fn test_fix_record_value_keeps_key() {
    RULE.assert_fixed_is("let x = { nothing: nothing }", "let x = { nothing: null }");
}

#[test]
fn test_fix_comparison_operand() {
    RULE.assert_fixed_is("if $x == nothing { 1 }", "if $x == null { 1 }");
}

#[test]
fn test_fix_statement_position() {
    RULE.assert_fixed_is("let x = nothing", "let x = null");
}
