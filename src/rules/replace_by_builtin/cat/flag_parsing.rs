use crate::{context::LintContext, rules::replace_by_builtin::cat::rule};

#[test]
fn converts_number_lines_flag() {
    let source = "^cat -n file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw file.txt | lines | enumerate"
        );
        assert!(
            fix.description.contains("enumerate") && fix.description.contains("-n"),
            "Fix should explain enumerate for line numbers: {}",
            fix.description
        );
    });
}

#[test]
fn converts_number_nonblank_flag() {
    let source = "^cat -b file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw file.txt | lines | enumerate | where $it.item != \"\""
        );
        assert!(
            fix.description.contains("non-blank") && fix.description.contains("enumerate"),
            "Fix should explain non-blank line numbering: {}",
            fix.description
        );
    });
}

#[test]
fn converts_show_ends_flag() {
    let source = "^cat -E file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw file.txt | lines"
        );
        assert!(
            fix.description.contains("-E") && fix.description.contains("line endings"),
            "Fix should explain -E flag: {}",
            fix.description
        );
    });
}

#[test]
fn converts_show_tabs_flag() {
    let source = "^cat -T file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw file.txt | lines"
        );
        assert!(
            fix.description.contains("-T") && fix.description.contains("tabs"),
            "Fix should explain -T flag: {}",
            fix.description
        );
    });
}

#[test]
fn converts_show_all_flag() {
    let source = "^cat -A file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open --raw file.txt | lines"
        );
        assert!(
            fix.description.contains("-E") || fix.description.contains("-T"),
            "Fix should explain -A encompasses -E and -T: {}",
            fix.description
        );
    });
}

#[test]
fn combines_number_with_multiple_files() {
    let source = "^cat -n file1.txt file2.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        // With flags, it processes each file separately
        assert!(
            fix.replacements[0].new_text.as_ref().contains("lines")
                && fix.replacements[0].new_text.as_ref().contains("enumerate"),
            "Should convert to lines with enumerate: {}",
            fix.replacements[0].new_text
        );
    });
}
