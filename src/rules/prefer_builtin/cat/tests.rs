use crate::{context::LintContext, rules::prefer_builtin::cat::rule};

#[test]
fn replaces_simple_cat_with_open_raw() {
    let source = "^cat file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "open --raw file.txt");
        assert!(
            fix.description.contains("auto-parse") || fix.description.contains("structured"),
            "Fix should mention auto-parsing advantage: {}",
            fix.description
        );
    });
}

#[test]
fn suggests_open_for_first_file_when_multiple() {
    let source = "^cat file1.txt file2.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw file1.txt"
        );
    });
}

#[test]
fn handles_structured_files() {
    let source = "^cat config.json";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw config.json"
        );
    });
}

#[test]
fn ignores_builtin_open() {
    let source = "open file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0);
    });
}
