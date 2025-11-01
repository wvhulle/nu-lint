use crate::{context::LintContext, rules::replace_by_builtin::find::rule};

#[test]
fn converts_iname_case_insensitive() {
    let source = r#"^find . -iname "*.TXT""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls ./**/*.TXT");
    });
}

#[test]
fn converts_pattern_without_wildcard() {
    let source = r#"^find . -name "test""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls ./**/*test*");
    });
}

#[test]
fn preserves_glob_pattern_with_wildcards() {
    let source = r#"^find src -name "test_*.rs""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls src/**/test_*.rs");
    });
}

#[test]
fn handles_path_with_spaces() {
    let source = r#"^find "my dir" -name "*.txt""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            r#"ls "my dir"/**/*.txt"#
        );
    });
}

#[test]
fn handles_absolute_path() {
    let source = r"^find /usr/local/bin -type f";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls /usr/local/bin/**/* | where type == file"
        );
    });
}
