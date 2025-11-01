use crate::{context::LintContext, rules::replace_by_builtin::head::rule};

#[test]
fn replaces_head_with_first() {
    let source = "^head -5 file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "first 5");
        assert!(
            fix.description.contains("cleaner syntax") || fix.description.contains("first"),
            "Fix should mention cleaner syntax: {}",
            fix.description
        );
    });
}

#[test]
fn handles_head_without_count() {
    let source = "^head file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "first 10");
    });
}
