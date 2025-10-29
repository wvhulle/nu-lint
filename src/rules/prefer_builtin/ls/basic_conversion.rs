use crate::{context::LintContext, rules::prefer_builtin::ls::rule};

#[test]
fn detects_external_ls() {
    let source = "^ls";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "prefer_builtin_ls");
    });
}

#[test]
fn replaces_simple_ls() {
    let source = "^ls";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls");
        assert!(
            fix.description.contains("structured") && fix.description.contains("data"),
            "Fix should mention structured data advantage: {}",
            fix.description
        );
    });
}

#[test]
fn preserves_directory_argument() {
    let source = "^ls /tmp";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls /tmp");
        assert!(
            fix.description.contains("structured"),
            "Fix should explain Nu's structured data advantage: {}",
            fix.description
        );
    });
}

#[test]
fn preserves_multiple_paths() {
    let source = "^ls src tests";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls src tests");
    });
}

#[test]
fn preserves_glob_pattern() {
    let source = "^ls *.rs";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls *.rs");
        assert!(
            fix.description.contains("structured"),
            "Fix should mention structured data: {}",
            fix.description
        );
    });
}

#[test]
fn detects_exa_command() {
    let source = "^exa";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls");
        assert!(
            fix.description.contains("exa") || fix.description.contains("structured"),
            "Fix should mention exa and structured data: {}",
            fix.description
        );
    });
}

#[test]
fn detects_eza_command() {
    let source = "^eza -la";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls --all");
    });
}

#[test]
fn ignores_builtin_ls() {
    let source = "ls";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn ignores_builtin_ls_with_args() {
    let source = "ls --all *.rs";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0);
    });
}
