use crate::{context::LintContext, rules::prefer_builtin::find::rule};

#[test]
fn ignores_unsupported_maxdepth_flag() {
    let source = r"^find . -maxdepth 2 -name '*.rs'";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(fix.replacements[0].new_text.as_ref().contains("*.rs"));
    });
}

#[test]
fn ignores_unsupported_executable_flag() {
    let source = r"^find . -executable";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls ./**/*");
    });
}

#[test]
fn handles_no_arguments() {
    let source = "^find";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls ./**/*");
    });
}
