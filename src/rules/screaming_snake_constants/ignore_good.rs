use super::rule;
use crate::LintContext;
#[test]
fn test_good_screaming_snake_const() {
    let good = "const MAX_RETRIES = 3";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_screaming_snake_with_underscores() {
    let good = "const DEFAULT_CONFIG_PATH = '/etc/config'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_screaming_snake_with_numbers() {
    let good = "const HTTP_STATUS_200 = 200";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_single_letter_const() {
    let good = "const PI = 3.14159";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_regular_variables() {
    let good = "let my_variable = 42";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_mutable_variables() {
    let good = "mut counter = 0";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_function_names() {
    let good = "def my-function [] { 'hello' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}
