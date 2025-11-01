use crate::{context::LintContext, rules::replace_by_builtin::sort::rule};

#[test]
fn replaces_simple_sort() {
    let source = "^sort file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort");
        assert!(
            fix.description.contains("any data type"),
            "Fix should mention data type flexibility: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_sort_in_pipeline() {
    let source = "ls | ^sort";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort");
    });
}

#[test]
fn ignores_builtin_sort() {
    let source = "ls | sort";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn ignores_builtin_sort_by() {
    let source = "ls | sort-by size";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0);
    });
}
