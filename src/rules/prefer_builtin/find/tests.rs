use crate::{context::LintContext, rules::prefer_builtin::find::rule};

#[test]
fn replaces_find_with_ls_glob() {
    let source = r#"^find . -name "*.rs""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls **/*.rs");
        assert!(
            fix.description.contains("glob") || fix.description.contains("ls"),
            "Fix should mention glob patterns: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_find_directory() {
    let source = "^find src";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls src/**/*");
    });
}
