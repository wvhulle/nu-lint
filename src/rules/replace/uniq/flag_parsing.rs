use crate::rules::replace::uniq::rule;

#[test]
fn converts_count_flag() {
    let source = "^uniq -c";
    rule().assert_replacement_contains(source, "uniq --count");
    rule().assert_fix_explanation_contains(source, "--count");
}

#[test]
fn converts_repeated_flag() {
    let source = "^uniq -d";
    rule().assert_replacement_contains(source, "uniq");
    rule().assert_fix_explanation_contains(source, "repeated");
    rule().assert_fix_explanation_contains(source, "count > 1");
}

#[test]
fn converts_unique_flag() {
    let source = "^uniq -u";
    rule().assert_replacement_contains(source, "uniq");
    rule().assert_fix_explanation_contains(source, "unique");
    rule().assert_fix_explanation_contains(source, "count == 1");
}

#[test]
fn converts_ignore_case_flag() {
    let source = "^uniq -i";
    rule().assert_replacement_contains(source, "uniq");
    rule().assert_fix_explanation_contains(source, "downcase");
}

#[test]
fn converts_skip_fields_flag() {
    let source = "^uniq -f 2";
    rule().assert_replacement_contains(source, "uniq");
    rule().assert_fix_explanation_contains(source, "uniq-by");
    rule().assert_fix_explanation_contains(source, "column");
}

#[test]
fn combines_count_with_other_flags() {
    let source = "^uniq -ci";
    rule().assert_replacement_contains(source, "uniq --count");
    rule().assert_fix_explanation_contains(source, "count");
    rule().assert_fix_explanation_contains(source, "case-insensitive");
}
