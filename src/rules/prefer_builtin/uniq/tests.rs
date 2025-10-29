use crate::{context::LintContext, rules::prefer_builtin::uniq::rule};

#[test]
fn replaces_uniq() {
    let source = "^uniq file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
        assert!(
            fix.description.contains("structured") || fix.description.contains("uniq-by"),
            "Fix should mention structured data support: {}",
            fix.description
        );
    });
}
