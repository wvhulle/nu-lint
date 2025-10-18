use super::rule;
use crate::LintContext;


#[test]
fn test_good_error_make() {
    let good = "error make { msg: 'Something went wrong', label: { text: 'here', span: $span } }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_error_make() {
    let good = "error make 'Invalid input'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_print_without_exit() {
    let good = "print 'This is just a message'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_conditional_error() {
    let good = "if $invalid { error make 'Invalid condition' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_exit_without_print() {
    let good = "exit 1";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_separate_operations() {
    let good = "print 'Processing...'; let result = some_command; exit 0";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}
