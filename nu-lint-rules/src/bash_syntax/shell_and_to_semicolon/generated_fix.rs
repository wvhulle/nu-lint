use super::RULE;

#[test]
fn fix_command_chain_to_semicolon() {
    RULE.assert_fixed_is("^make && ^make test", "^make; ^make test");
}

#[test]
fn fix_keeps_line_break_after_operator() {
    RULE.assert_fixed_is("^make &&\n^make test", "^make;\n^make test");
}

#[test]
fn fix_operator_starting_a_line() {
    RULE.assert_fixed_is("&& ^make test", "; ^make test");
}

#[test]
fn fix_pipeline_command_chain_to_semicolon() {
    RULE.assert_fixed_is("^cat a.txt | ^wc && ^pwd", "^cat a.txt | ^wc; ^pwd");
}

#[test]
fn fix_boolean_in_if_to_and() {
    RULE.assert_fixed_is(
        "if $env.A && $env.B { ^make }",
        "if $env.A and $env.B { ^make }",
    );
}

#[test]
fn fix_boolean_in_where_to_and() {
    RULE.assert_fixed_is(
        "ls | where $it.size > 1kb && $it.name != a",
        "ls | where $it.size > 1kb and $it.name != a",
    );
}

#[test]
fn fix_boolean_in_parens_to_and() {
    RULE.assert_fixed_is("let both = (true && false)", "let both = (true and false)");
}
