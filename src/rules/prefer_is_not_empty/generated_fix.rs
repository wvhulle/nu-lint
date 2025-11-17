use super::rule;

#[test]
fn test_prefer_is_not_empty_fix_simple() {
    let bad_code = "if not ($list | is-empty) { echo 'has items' }";
    rule().assert_replacement_contains(bad_code, "$list | is-not-empty");
}

#[test]
fn test_prefer_is_not_empty_fix_variable() {
    let bad_code = "let has_data = not ($data | is-empty)";
    rule().assert_replacement_contains(bad_code, "$data | is-not-empty");
}

#[test]
fn test_prefer_is_not_empty_fix_complex_expr() {
    let bad_code = "if not ($items | filter {|x| $x > 5} | is-empty) { echo 'found' }";
    rule().assert_replacement_contains(bad_code, "| is-not-empty");
    rule().assert_replacement_contains(bad_code, "filter");
}

#[test]
fn test_prefer_is_not_empty_fix_description() {
    let bad_code = "not ($list | is-empty)";
    rule().assert_fix_explanation_contains(bad_code, "Replace");
    rule().assert_fix_explanation_contains(bad_code, "is-not-empty");
}

#[test]
fn test_prefer_is_not_empty_fix_multiple_patterns() {
    let bad_code = r#"
if not ($list | is-empty) and not ($other | is-empty) {
    echo "both not empty"
}
"#;
    rule().assert_count(bad_code, 2);
    rule().assert_replacement_contains(bad_code, "| is-not-empty");
}
