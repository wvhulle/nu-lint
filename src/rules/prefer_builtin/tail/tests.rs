use crate::{context::LintContext, rules::prefer_builtin::tail::rule};

#[test]
fn replaces_tail_with_last() {
    let source = "^tail -10 file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "last 10");
        assert!(
            fix.description.contains("cleaner syntax") || fix.description.contains("last"),
            "Fix should mention cleaner syntax: {}",
            fix.description
        );
    });
}
