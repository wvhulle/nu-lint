use super::rule;
use crate::LintContext;
#[test]
fn test_good_parse_command() {
    let good = "'name:john age:30' | parse '{name}:{age}'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_parse_with_patterns() {
    let good = "'User: alice, ID: 123' | parse 'User: {name}, ID: {id}'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_simple_split() {
    let good = "'a,b,c' | split row ','";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_split_for_iteration() {
    let good = "'a,b,c' | split row ',' | each { |item| $item | str upcase }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_split_column() {
    let good = "'name,age,city' | split column ',' name age city";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_from_csv() {
    let good = "'name,age\njohn,30\njane,25' | from csv";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}
