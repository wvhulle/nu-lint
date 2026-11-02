use super::RULE;

#[test]
fn detect_command_fallback() {
    RULE.assert_detects("^make || ^echo failed");
}

#[test]
fn detect_chain_once_per_statement() {
    RULE.assert_count("^make || ^make retry || ^echo failed", 1);
}

#[test]
fn detect_each_boolean_operator() {
    RULE.assert_count("if $env.A || $env.B || $env.C { ^make }", 2);
}

#[test]
fn detect_operator_in_nested_block() {
    RULE.assert_detects("def build [] { ^make || ^echo failed }");
}

#[test]
fn detect_bash_test_without_fix() {
    RULE.assert_detects_without_fix("[ -f file.txt ] || ^touch file.txt");
}

#[test]
fn detect_dangling_operator_without_fix() {
    RULE.assert_detects_without_fix("^make ||");
}
