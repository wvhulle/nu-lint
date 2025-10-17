use super::*;
use crate::rules::prefer_error_make::PreferErrorMake;

#[test]
fn test_good_error_make() {
    let rule = PreferErrorMake;
    let good = "error make { msg: 'Something went wrong', label: { text: 'here', span: $span } }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_error_make() {
    let rule = PreferErrorMake;
    let good = "error make 'Invalid input'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_print_without_exit() {
    let rule = PreferErrorMake;
    let good = "print 'This is just a message'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_conditional_error() {
    let rule = PreferErrorMake;
    let good = "if $invalid { error make 'Invalid condition' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_exit_without_print() {
    let rule = PreferErrorMake;
    let good = "exit 1";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_separate_operations() {
    let rule = PreferErrorMake;
    let good = "print 'Processing...'; let result = some_command; exit 0";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
