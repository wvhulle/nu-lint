use crate::{context::LintContext, rules::prefer_builtin::sort::rule};

#[test]
fn replaces_sort() {
    let source = "^sort file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort");
        assert!(
            fix.description.contains("any data type") || fix.description.contains("natural"),
            "Fix should mention data type flexibility: {}",
            fix.description
        );
    });
}
