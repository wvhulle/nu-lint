use super::rule;
use crate::LintContext;

#[test]
fn test_good_compound_add_assignment() {
    let good = "mut x = 5; $x += 3";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_compound_subtract_assignment() {
    let good = "mut count = 10; $count -= 2";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_compound_multiply_assignment() {
    let good = "mut factor = 2; $factor *= 3";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_assignment() {
    let good = "mut x = 5; $x = 10";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_different_variables() {
    let good = "mut x = 5; mut y = 3; $x = $y + 2";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_append_assignment() {
    let good = "mut items = []; $items ++= [1, 2, 3]";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}
