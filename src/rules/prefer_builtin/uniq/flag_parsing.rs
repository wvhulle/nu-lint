use crate::{context::LintContext, rules::prefer_builtin::uniq::rule};

#[test]
fn converts_count_flag() {
    let source = "^uniq -c";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq --count");
        assert!(
            fix.description.contains("--count"),
            "Fix should explain count flag: {}",
            fix.description
        );
    });
}

#[test]
fn converts_repeated_flag() {
    let source = "^uniq -d";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
        assert!(
            fix.description.contains("repeated") && fix.description.contains("count > 1"),
            "Fix should suggest uniq --count | where count > 1: {}",
            fix.description
        );
    });
}

#[test]
fn converts_unique_flag() {
    let source = "^uniq -u";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
        assert!(
            fix.description.contains("unique") && fix.description.contains("count == 1"),
            "Fix should suggest uniq --count | where count == 1: {}",
            fix.description
        );
    });
}

#[test]
fn converts_ignore_case_flag() {
    let source = "^uniq -i";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
        assert!(
            fix.description.contains("case-insensitive") || fix.description.contains("downcase"),
            "Fix should suggest str downcase: {}",
            fix.description
        );
    });
}

#[test]
fn converts_skip_fields_flag() {
    let source = "^uniq -f 2";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
        assert!(
            fix.description.contains("uniq-by") && fix.description.contains("column"),
            "Fix should suggest uniq-by for field-based deduplication: {}",
            fix.description
        );
    });
}

#[test]
fn combines_count_with_other_flags() {
    let source = "^uniq -ci";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq --count");
        assert!(
            fix.description.contains("count") && fix.description.contains("case-insensitive"),
            "Fix should mention both flags: {}",
            fix.description
        );
    });
}
