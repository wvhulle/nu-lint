use super::RULE;
#[test]
fn ignore_screaming_snake_const() {
    let good = "const MAX_RETRIES = 3";
    RULE.assert_ignores(good);
}

#[test]
fn ignore_multi_word_screaming_snake_const() {
    let good = "const DEFAULT_CONFIG_PATH = '/etc/config'";
    RULE.assert_ignores(good);
}

#[test]
fn ignore_screaming_snake_with_numbers() {
    let good = "const HTTP_STATUS_200 = 200";
    RULE.assert_ignores(good);
}

#[test]
fn ignore_single_letter_uppercase_const() {
    let good = "const PI = 3.14159";
    RULE.assert_ignores(good);
}

#[test]
fn ignore_let_variables() {
    let good = "let my_variable = 42";
    RULE.assert_ignores(good);
}

#[test]
fn ignore_mut_variables() {
    let good = "mut counter = 0";
    RULE.assert_ignores(good);
}

#[test]
fn ignore_function_definitions() {
    let good = "def my-function [] { 'hello' }";
    RULE.assert_ignores(good);
}
