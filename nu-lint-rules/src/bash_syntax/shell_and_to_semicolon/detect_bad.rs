use super::RULE;

#[test]
fn detect_every_operator_in_chain() {
    RULE.assert_count("^make && ^make test && ^make install", 2);
}

#[test]
fn detect_operators_on_separate_statements() {
    RULE.assert_count("^make && ^make test\n^ls && ^pwd", 2);
}

#[test]
fn detect_operator_inside_closure() {
    RULE.assert_detects("[1 2] | each {|number| ^make && ^make test }");
}

#[test]
fn detect_bash_test_without_fix() {
    RULE.assert_detects_without_fix("[ -f file.txt ] && ^cat file.txt");
}
