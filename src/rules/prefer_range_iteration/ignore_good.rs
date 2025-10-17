use super::*;
use crate::rules::prefer_range_iteration::PreferRangeIteration;

#[test]
fn test_good_range_iteration() {
    let rule = PreferRangeIteration;
    let good = "0..10 | each { |i| print $i }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_for_range() {
    let rule = PreferRangeIteration;
    let good = "for i in 0..5 { print $i }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_list_iteration() {
    let rule = PreferRangeIteration;
    let good = "[1, 2, 3] | each { |item| $item * 2 }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_while_different_purpose() {
    let rule = PreferRangeIteration;
    let good = "mut running = true; while $running { $running = check_condition }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_counter() {
    let rule = PreferRangeIteration;
    let good = "mut count = 0";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_generate_sequence() {
    let rule = PreferRangeIteration;
    let good = "seq 1 10 | each { |n| $n * $n }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_range_with_step() {
    let rule = PreferRangeIteration;
    let good = "0..10 | where { |x| $x mod 2 == 0 } | each { |n| $n * 2 }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_enumerate_pattern() {
    let rule = PreferRangeIteration;
    let good = "['a', 'b', 'c'] | enumerate | each { |item| $'($item.index): ($item.item)' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_range_with_reduce() {
    let rule = PreferRangeIteration;
    let good = "1..5 | reduce { |it, acc| $acc + $it }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
