use crate::{context::LintContext, rules::prefer_builtin::grep::rule};

#[test]
fn replaces_simple_grep_with_find() {
    let source = r#"^grep "pattern""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), r#"find ""pattern"""#);
        assert!(
            fix.description.contains("case-insensitive") && fix.description.contains("default"),
            "Fix should explicitly mention case-insensitive is default in Nu: {}",
            fix.description
        );
    });
}

#[test]
fn mentions_redundant_i_flag() {
    let source = r#"^grep -i "warning" logs.txt"#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.description.contains("redundant") && fix.description.contains("-i"),
            "Fix must explicitly state that -i flag is redundant in Nu: {}",
            fix.description
        );
    });
}

#[test]
fn suggests_where_for_complex_grep() {
    let source = r#"^grep -r "TODO" ."#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            r#"where $it =~ "pattern""#
        );
        assert!(
            fix.description.contains("regex") || fix.description.contains("where"),
            "Fix should explain when to use where for filtering: {}",
            fix.description
        );
    });
}

#[test]
fn mentions_structured_data_advantage() {
    let source = r#"^grep "error""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.description.contains("structured") || fix.description.contains("data"),
            "Fix should mention that find works on structured data: {}",
            fix.description
        );
    });
}
