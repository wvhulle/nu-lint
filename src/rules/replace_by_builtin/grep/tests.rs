use crate::rules::replace_by_builtin::grep::rule;

#[test]
fn replaces_simple_grep_with_find() {
    let source = r#"^grep "pattern""#;
    rule().assert_fix_contains(source, r#"find "pattern""#);
    rule().assert_fix_explanation_contains(source, "case-insensitive");
    rule().assert_fix_explanation_contains(source, "default");
}

#[test]
fn mentions_redundant_i_flag() {
    let source = r#"^grep -i "warning" logs.txt"#;
    rule().assert_fix_contains(source, r#"open logs.txt | lines | where $it =~ "warning""#);
    rule().assert_fix_explanation_contains(source, "redundant");
    rule().assert_fix_explanation_contains(source, "-i");
}

#[test]
fn suggests_where_for_complex_grep() {
    let source = r#"^grep -r "TODO" ."#;
    rule().assert_fix_contains(source, r#"open . | lines | where $it =~ "TODO""#);
    rule().assert_fix_explanation_contains(source, "where");
}

#[test]
fn mentions_structured_data_advantage() {
    let source = r#"^grep "error""#;
    rule().assert_fix_contains(source, r#"find "error""#);
    rule().assert_fix_explanation_contains(source, "structured");
}
