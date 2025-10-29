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
fn handles_multiple_files() {
    let source = "^cat file1.txt file2.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "[file1.txt file2.txt] | each {|f| open --raw $f} | str join"
        );
        assert!(
            fix.description.contains("each") && fix.description.contains("multiple"),
            "Fix should explain multiple file handling: {}",
            fix.description
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
fn detects_tac_command() {
    let source = "^tac file.log";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "open --raw file.log");
        assert!(
            fix.description.contains("reverse") || fix.description.contains("lines"),
            "Fix should suggest using reverse for tac: {}",
            fix.description
        );
    });
}

#[test]
fn detects_more_command() {
    let source = "^more documentation.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw documentation.txt"
        );
    });
}

#[test]
fn detects_less_command() {
    let source = "^less output.log";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw output.log"
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
