use super::RULE;

#[test]
fn fix_command_fallback_to_try_catch() {
    RULE.assert_fixed_is(
        "^make || ^echo failed",
        "try { ^make } catch { ^echo failed }",
    );
}

#[test]
fn fix_chain_to_nested_try_catch() {
    RULE.assert_fixed_is(
        "^make || ^make retry || ^echo failed",
        "try { ^make } catch { try { ^make retry } catch { ^echo failed } }",
    );
}

#[test]
fn fix_wraps_whole_pipeline() {
    RULE.assert_fixed_is(
        "^cat a.txt | ^wc || ^echo failed",
        "try { ^cat a.txt | ^wc } catch { ^echo failed }",
    );
}

#[test]
fn fix_stays_inside_enclosing_block() {
    RULE.assert_fixed_is(
        "def build [] { ^make || ^echo failed }",
        "def build [] { try { ^make } catch { ^echo failed } }",
    );
}

#[test]
fn fix_boolean_in_if_to_or() {
    RULE.assert_fixed_is(
        "if $env.A || $env.B { ^make }",
        "if $env.A or $env.B { ^make }",
    );
}
