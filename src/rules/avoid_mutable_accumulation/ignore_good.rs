use super::*;
use crate::rules::avoid_mutable_accumulation::AvoidMutableAccumulation;

#[test]
fn test_functional_each_pipeline() {
    let rule = AvoidMutableAccumulation;
    let good = "[1, 2, 3] | each { |x| $x * 2 }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_functional_where_filter() {
    let rule = AvoidMutableAccumulation;
    let good = "[1, 2, 3, 4] | where { |x| $x > 2 }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_immutable_list() {
    let rule = AvoidMutableAccumulation;
    let good = "let items = [1, 2, 3]";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_functional_reduce() {
    let rule = AvoidMutableAccumulation;
    let good = "[1, 2, 3] | reduce { |it, acc| $acc + $it }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_mutable_without_append() {
    let rule = AvoidMutableAccumulation;
    let good = "mut counter = 0";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
