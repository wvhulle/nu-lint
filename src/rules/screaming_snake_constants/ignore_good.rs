use super::*;
use crate::rules::screaming_snake_constants::ScreamingSnakeConstants;

#[test]
fn test_good_screaming_snake_const() {
    let rule = ScreamingSnakeConstants;
    let good = "const MAX_RETRIES = 3";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_screaming_snake_with_underscores() {
    let rule = ScreamingSnakeConstants;
    let good = "const DEFAULT_CONFIG_PATH = '/etc/config'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_screaming_snake_with_numbers() {
    let rule = ScreamingSnakeConstants;
    let good = "const HTTP_STATUS_200 = 200";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_single_letter_const() {
    let rule = ScreamingSnakeConstants;
    let good = "const PI = 3.14159";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_regular_variables() {
    let rule = ScreamingSnakeConstants;
    let good = "let my_variable = 42";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_mutable_variables() {
    let rule = ScreamingSnakeConstants;
    let good = "mut counter = 0";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_function_names() {
    let rule = ScreamingSnakeConstants;
    let good = "def my-function [] { 'hello' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
