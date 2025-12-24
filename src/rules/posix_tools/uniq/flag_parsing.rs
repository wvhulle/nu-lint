use super::RULE;

#[test]
fn converts_count_flag() {
    let source = "^uniq -c";
    RULE.assert_replacement_contains(source, "uniq --count");
    RULE.assert_fix_explanation_contains(source, "--count");
}

#[test]
fn converts_repeated_flag() {
    let source = "^uniq -d";
    RULE.assert_replacement_contains(source, "uniq");
    RULE.assert_fix_explanation_contains(source, "repeated");
    RULE.assert_fix_explanation_contains(source, "count > 1");
}

#[test]
fn converts_unique_flag() {
    let source = "^uniq -u";
    RULE.assert_replacement_contains(source, "uniq");
    RULE.assert_fix_explanation_contains(source, "unique");
    RULE.assert_fix_explanation_contains(source, "count == 1");
}

#[test]
fn converts_ignore_case_flag() {
    let source = "^uniq -i";
    RULE.assert_replacement_contains(source, "uniq");
    RULE.assert_fix_explanation_contains(source, "downcase");
}

#[test]
fn converts_skip_fields_flag() {
    let source = "^uniq -f 2";
    RULE.assert_replacement_contains(source, "uniq");
    RULE.assert_fix_explanation_contains(source, "uniq-by");
    RULE.assert_fix_explanation_contains(source, "column");
}

#[test]
fn combines_count_with_other_flags() {
    let source = "^uniq -ci";
    RULE.assert_replacement_contains(source, "uniq --count");
    RULE.assert_fix_explanation_contains(source, "count");
    RULE.assert_fix_explanation_contains(source, "case-insensitive");
}
