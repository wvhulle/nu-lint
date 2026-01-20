use super::RULE;

#[test]
fn test_prefer_is_not_empty_fix_simple() {
    let bad_code = "if not ($list | is-empty) { echo 'has items' }";
    RULE.assert_fixed_contains(bad_code, "$list | is-not-empty");
}

#[test]
fn test_prefer_is_not_empty_fix_variable() {
    let bad_code = "let has_data = not ($data | is-empty)";
    RULE.assert_fixed_contains(bad_code, "$data | is-not-empty");
}

#[test]
fn test_prefer_is_not_empty_fix_complex_expr() {
    let bad_code = "if not ($items | filter {|x| $x > 5} | is-empty) { echo 'found' }";
    RULE.assert_fixed_contains(bad_code, "| is-not-empty");
    RULE.assert_fixed_contains(bad_code, "filter");
}

#[test]
fn test_prefer_is_not_empty_fix_multiple_patterns() {
    let bad_code = r#"
if not ($list | is-empty) and not ($other | is-empty) {
    echo "both not empty"
}
"#;
    RULE.assert_count(bad_code, 2);
    RULE.assert_fixed_contains(bad_code, "| is-not-empty");
}

#[test]
fn test_prefer_is_not_empty_fix_preserves_parentheses() {
    // Bug fix: the fix should preserve parentheses for correct precedence
    // `if not ($path | is-empty)` should become `if ($path | is-not-empty)`, not
    // `if $path | is-not-empty`
    let bad_code = "if not ($path | is-empty) { }";
    RULE.assert_fixed_contains(bad_code, "($path | is-not-empty)");
}

#[test]
fn test_prefer_is_not_empty_fix_parentheses_in_condition() {
    // Ensure parentheses are added in condition context
    let bad_code = "let result = not ($data | is-empty)";
    RULE.assert_fixed_contains(bad_code, "($data | is-not-empty)");
}
