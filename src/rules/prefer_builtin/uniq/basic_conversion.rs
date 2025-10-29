use crate::{context::LintContext, rules::prefer_builtin::uniq::rule};

#[test]
fn replaces_simple_uniq() {
    let source = "^uniq";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
        assert!(
            fix.description.contains("structured data"),
            "Fix should mention structured data: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_uniq_in_pipeline() {
    let source = "ls | ^uniq";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
    });
}

#[test]
fn ignores_builtin_uniq() {
    let source = "ls | uniq";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn ignores_builtin_uniq_by() {
    let source = "ls | uniq-by name";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0);
    });
}
