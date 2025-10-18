use super::rule;
use crate::LintContext;

#[test]
fn test_good_nu_complete_prefix() {
    let good = "def 'nu-complete git-branch' [] { git branch | lines }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_nu_complete_with_spaces() {
    let good = "def 'nu-complete file types' [] { ['txt', 'md', 'rs'] }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_non_completion_function() {
    let good = "def process-data [] { echo 'processing' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_function() {
    let good = "def hello [name: string] { $'Hello ($name)!' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_complex_function() {
    let good = "def calculate-sum [numbers: list<int>] { $numbers | math sum }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}
