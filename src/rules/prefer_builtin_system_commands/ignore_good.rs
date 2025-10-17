use super::*;
use crate::rules::prefer_builtin_system_commands::AvoidExternalSystemTools;

#[test]
fn test_good_builtin_env() {
    let rule = AvoidExternalSystemTools;
    let good = "$env.HOME";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_builtin_date() {
    let rule = AvoidExternalSystemTools;
    let good = "date now";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_builtin_whoami() {
    let rule = AvoidExternalSystemTools;
    let good = "whoami";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_builtin_sys_host() {
    let rule = AvoidExternalSystemTools;
    let good = "(sys host).hostname";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_builtin_help() {
    let rule = AvoidExternalSystemTools;
    let good = "help ls";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_builtin_which() {
    let rule = AvoidExternalSystemTools;
    let good = "which nu";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_builtin_input() {
    let rule = AvoidExternalSystemTools;
    let good = "let name = input 'Enter your name: '";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
