use super::*;
use crate::rules::prefer_builtin_text_transforms::AvoidExternalTextTools;

#[test]
fn test_good_str_replace() {
    let rule = AvoidExternalTextTools;
    let good = "'hello world' | str replace 'world' 'universe'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_select_columns() {
    let rule = AvoidExternalTextTools;
    let good = "ls | select name size";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_where_filter() {
    let rule = AvoidExternalTextTools;
    let good = "ls | where size > 1000";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_length_count() {
    let rule = AvoidExternalTextTools;
    let good = "ls | length";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_str_length() {
    let rule = AvoidExternalTextTools;
    let good = "'hello' | str length";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_tee_save() {
    let rule = AvoidExternalTextTools;
    let good = "ls | tee { save list.json }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_str_case_conversion() {
    let rule = AvoidExternalTextTools;
    let good = "'HELLO' | str downcase";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
