use super::*;
use crate::rules::prefer_builtin_system_commands::AvoidExternalSystemTools;

#[test]
fn test_good_builtin_env_access() {
    let rule = AvoidExternalSystemTools;
    let good = "$env.HOME";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_env_with_fallback() {
    let rule = AvoidExternalSystemTools;
    let good = "$env.EDITOR? | default 'vim'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_env_in_pipeline() {
    let rule = AvoidExternalSystemTools;
    let good = "echo $env.PATH | split row (char esep)";
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
fn test_good_date_formatting() {
    let rule = AvoidExternalSystemTools;
    let good = "date now | format date '%Y-%m-%d'";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_date_into_conversion() {
    let rule = AvoidExternalSystemTools;
    let good = "'2024-01-01' | into datetime";
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
fn test_good_sys_full_info() {
    let rule = AvoidExternalSystemTools;
    let good = "sys host | select name kernel_version";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_sys_net_for_ip() {
    let rule = AvoidExternalSystemTools;
    let good = "sys net | where name == 'eth0' | get ip.0";
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

#[test]
fn test_good_input_secure() {
    let rule = AvoidExternalSystemTools;
    let good = "let password = input -s 'Password: '";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_cd_with_tilde() {
    let rule = AvoidExternalSystemTools;
    let good = "cd ~/projects";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    });
}
