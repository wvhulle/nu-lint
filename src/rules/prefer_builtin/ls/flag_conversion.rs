use crate::{context::LintContext, rules::prefer_builtin::ls::rule};

#[test]
fn converts_all_flag() {
    let source = "^ls -a";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls --all");
        assert!(
            fix.description.contains("--all"),
            "Fix should mention --all flag: {}",
            fix.description
        );
    });
}

#[test]
fn converts_combined_flags() {
    let source = "^ls -la";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls --all");
        assert!(
            fix.description.contains("-l") && fix.description.contains("not needed"),
            "Fix should mention that -l flag is not needed: {}",
            fix.description
        );
    });
}

#[test]
fn converts_human_readable_flag() {
    let source = "^ls -h";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls");
        assert!(
            fix.description.contains("-h") && fix.description.contains("not needed"),
            "Fix should mention that -h is not needed: {}",
            fix.description
        );
    });
}

#[test]
fn converts_long_flag() {
    let source = "^ls -l";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls");
        assert!(
            fix.description.contains("-l") && fix.description.contains("not needed"),
            "Fix should explain -l is unnecessary: {}",
            fix.description
        );
    });
}

#[test]
fn converts_long_format_all_flag() {
    let source = "^ls --all";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls --all");
    });
}

#[test]
fn mentions_recursive_alternative() {
    let source = "^ls -R";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.description.contains("recursive") && fix.description.contains("glob"),
            "Fix should suggest glob patterns for recursion: {}",
            fix.description
        );
    });
}
