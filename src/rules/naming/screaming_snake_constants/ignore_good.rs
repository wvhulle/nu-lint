use super::rule;
#[test]
fn test_good_screaming_snake_const() {
    let good = "const MAX_RETRIES = 3";
    rule().assert_ignores(good);
}

#[test]
fn test_good_screaming_snake_with_underscores() {
    let good = "const DEFAULT_CONFIG_PATH = '/etc/config'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_screaming_snake_with_numbers() {
    let good = "const HTTP_STATUS_200 = 200";
    rule().assert_ignores(good);
}

#[test]
fn test_good_single_letter_const() {
    let good = "const PI = 3.14159";
    rule().assert_ignores(good);
}

#[test]
fn test_good_regular_variables() {
    let good = "let my_variable = 42";
    rule().assert_ignores(good);
}

#[test]
fn test_good_mutable_variables() {
    let good = "mut counter = 0";
    rule().assert_ignores(good);
}

#[test]
fn test_good_function_names() {
    let good = "def my-function [] { 'hello' }";
    rule().assert_ignores(good);
}
