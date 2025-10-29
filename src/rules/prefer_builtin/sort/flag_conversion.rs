use crate::{context::LintContext, rules::prefer_builtin::sort::rule};

#[test]
fn converts_reverse_flag() {
    let source = "^sort -r";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort --reverse");
        assert!(
            fix.description.contains("--reverse"),
            "Fix should explain reverse flag: {}",
            fix.description
        );
    });
}

#[test]
fn converts_numeric_flag() {
    let source = "^sort -n";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort --natural");
        assert!(
            fix.description.contains("natural") && fix.description.contains("numeric"),
            "Fix should explain natural sorting: {}",
            fix.description
        );
    });
}

#[test]
fn converts_unique_flag() {
    let source = "^sort -u";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort");
        assert!(
            fix.description.contains("uniq") && fix.description.contains("-u"),
            "Fix should suggest using uniq for unique: {}",
            fix.description
        );
    });
}

#[test]
fn converts_key_field() {
    let source = "^sort -k 2";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort-by 2");
        assert!(
            fix.description.contains("sort-by") && fix.description.contains("column"),
            "Fix should explain sort-by for column sorting: {}",
            fix.description
        );
    });
}

#[test]
fn converts_key_field_compact_format() {
    let source = "^sort -k2";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort-by 2");
    });
}

#[test]
fn converts_ignore_case_flag() {
    let source = "^sort -f";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort");
        assert!(
            fix.description.contains("case-insensitive") || fix.description.contains("downcase"),
            "Fix should explain case-insensitive sorting: {}",
            fix.description
        );
    });
}

#[test]
fn combines_reverse_and_numeric() {
    let source = "^sort -nr";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "sort --natural --reverse"
        );
        assert!(
            fix.description.contains("natural") && fix.description.contains("reverse"),
            "Fix should explain both flags: {}",
            fix.description
        );
    });
}

#[test]
fn combines_key_and_reverse() {
    let source = "^sort -k 3 -r";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "sort-by 3 --reverse");
    });
}
