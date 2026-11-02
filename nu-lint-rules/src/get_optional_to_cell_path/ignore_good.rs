use super::RULE;

#[test]
fn ignore_optional_cell_path() {
    RULE.assert_ignores("let sample = {a: 1}\n$sample.a?");
}

#[test]
fn ignore_required_get() {
    RULE.assert_ignores("let sample = {a: 1}\n$sample | get a");
}

#[test]
fn ignore_numeric_index() {
    RULE.assert_ignores("let sample = [1 2]\n$sample | get -o 0");
}

#[test]
fn ignore_numeric_member_in_path() {
    RULE.assert_ignores("let sample = {a: [1]}\n$sample | get -o a.0");
}

#[test]
fn ignore_dynamic_key() {
    RULE.assert_ignores("let sample = {a: 1}\nlet key = 'a'\n$sample | get -o $key");
}

#[test]
fn ignore_key_check_with_is_not_empty() {
    RULE.assert_ignores("let sample = {a: 1}\n$sample | get -o a | is-not-empty");
}

#[test]
fn ignore_key_check_with_is_empty() {
    RULE.assert_ignores("let sample = {a: 1}\n$sample | get -o a | is-empty");
}

#[test]
fn ignore_chained_gets() {
    RULE.assert_ignores("let sample = {a: {b: 1}}\n$sample | get -o a | get -o b");
}

#[test]
fn ignore_non_variable_input() {
    RULE.assert_ignores("ls | get -o name");
}

#[test]
fn ignore_dollar_in_input() {
    RULE.assert_ignores("def read-name []: record -> any { $in | get -o name }");
}

#[test]
fn ignore_multiple_paths() {
    RULE.assert_ignores("let sample = {a: 1 b: 2}\n$sample | get -o a b");
}

#[test]
fn ignore_user_defined_get() {
    RULE.assert_ignores(
        "def get [path: cell-path, --optional(-o)] { $in }\nlet sample = {a: 1}\n$sample | get -o \
         a",
    );
}
