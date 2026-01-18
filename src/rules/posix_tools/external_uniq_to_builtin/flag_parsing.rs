use super::RULE;

#[test]
fn converts_count_flag() {
    let source = "^uniq -c";
    RULE.assert_fixed_contains(source, "uniq --count");
}

#[test]
fn converts_repeated_flag() {
    let source = "^uniq -d";
    RULE.assert_fixed_contains(source, "uniq");
}

#[test]
fn converts_unique_flag() {
    let source = "^uniq -u";
    RULE.assert_fixed_contains(source, "uniq");
}

#[test]
fn converts_ignore_case_flag() {
    let source = "^uniq -i";
    RULE.assert_fixed_contains(source, "uniq");
}

#[test]
fn converts_skip_fields_flag() {
    let source = "^uniq -f 2";
    RULE.assert_fixed_contains(source, "uniq");
}

#[test]
fn combines_count_with_other_flags() {
    let source = "^uniq -ci";
    RULE.assert_fixed_contains(source, "uniq --count");
}
