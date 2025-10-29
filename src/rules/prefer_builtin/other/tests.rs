use crate::{context::LintContext, rules::prefer_builtin::other::rule};

#[test]
fn replaces_env_variable_access() {
    let source = "^printenv HOME";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "$env.HOME");
        assert!(
            fix.description.contains("directly"),
            "Fix should explain direct access: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_date_with_date_now() {
    let source = "^date";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "date now");
        assert!(
            fix.description.contains("datetime") || fix.description.contains("timezone"),
            "Fix should mention datetime features: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_hostname_with_sys_host() {
    let source = "^hostname";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "(sys host).hostname");
        assert!(
            fix.description.contains("sys"),
            "Fix should mention sys command: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_man_with_help() {
    let source = "^man ls";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "help ls");
    });
}

#[test]
fn replaces_which() {
    let source = "^which python";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "which python");
    });
}

#[test]
fn replaces_read_with_input() {
    let source = "^read";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "input");
    });
}

#[test]
fn replaces_read_silent_with_input_s() {
    let source = "^read -s";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "input -s");
        assert!(
            fix.description.contains("password") || fix.description.contains("hidden"),
            "Fix should mention secure input: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_echo_with_print() {
    let source = "^echo hello";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "print hello");
    });
}

#[test]
fn replaces_wc_lines() {
    let source = "^wc -l";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "lines | length");
        assert!(
            fix.description.contains("count"),
            "Fix should mention counting: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_awk_with_pipeline() {
    let source = "^awk";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "where | select | each"
        );
        assert!(
            fix.description.contains("pipeline"),
            "Fix should mention data pipeline: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_cut_with_select() {
    let source = "^cut";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "select");
        assert!(
            fix.description.contains("columns"),
            "Fix should mention column selection: {}",
            fix.description
        );
    });
}
