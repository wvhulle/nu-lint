use super::*;
use crate::rules::max_positional_params::MaxPositionalParams;

#[test]
fn test_function_with_no_params() {
    let rule = MaxPositionalParams::new();
    let good = "def hello [] { 'Hello World!' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_function_with_one_param() {
    let rule = MaxPositionalParams::new();
    let good = "def greet [name: string] { $'Hello ($name)!' }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_function_with_two_params() {
    let rule = MaxPositionalParams::new();
    let good = "def add [a: int, b: int] { $a + $b }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_function_with_flags() {
    let rule = MaxPositionalParams::new();
    let good = "def process [input: string, --verbose (-v), --output (-o): string] { echo $input }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_function_with_optional_param() {
    let rule = MaxPositionalParams::new();
    let good = "def calc [x: int, y?: int] { $x + ($y | default 0) }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
