use super::rule;
use crate::LintContext;


#[test]
fn test_good_match_statement() {
    let good = "match $status { 'ok' => 'success', 'error' => 'failed', _ => 'unknown' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_if() {
    let good = "if $condition { 'yes' } else { 'no' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_different_variables() {
    let good = "if $x == 1 { 'one' } else if $y == 2 { 'two' } else { 'other' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_complex_conditions() {
    let good = "if $x > 5 and $y < 10 { 'range' } else { 'outside' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_match_with_guard() {
    let good = "match $value { x if $x > 10 => 'big', x if $x > 5 => 'medium', _ => 'small' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_nested_if() {
    let good = "if $outer { if $inner { 'both' } else { 'outer' } } else { 'neither' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}
