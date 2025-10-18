use super::rule;
use crate::LintContext;

#[test]
fn test_good_range_iteration() {
    let good = "0..10 | each { |i| print $i }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_for_range() {
    let good = "for i in 0..5 { print $i }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_list_iteration() {
    let good = "[1, 2, 3] | each { |item| $item * 2 }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_while_different_purpose() {
    let good = "mut running = true; while $running { $running = check_condition }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_counter() {
    let good = "mut count = 0";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_generate_sequence() {
    let good = "seq 1 10 | each { |n| $n * $n }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_range_with_step() {
    let good = "0..10 | where { |x| $x mod 2 == 0 } | each { |n| $n * 2 }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_enumerate_pattern() {
    let good = "['a', 'b', 'c'] | enumerate | each { |item| $'($item.index): ($item.item)' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_range_with_reduce() {
    let good = "1..5 | reduce { |it, acc| $acc + $it }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}
