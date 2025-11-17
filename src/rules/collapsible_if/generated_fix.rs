use super::rule;

#[test]
fn test_fix_simple_nested_if() {
    let bad_code = r#"if $x { if $y { print "yes" } }"#;
    rule().assert_replacement_contains(bad_code, r#"if $x and $y { print "yes" }"#);
}

#[test]
fn test_fix_nested_with_complex_conditions() {
    let bad_code = r#"if $x > 10 { if $y < 20 { echo "range" } }"#;
    rule().assert_replacement_contains(bad_code, r#"if $x > 10 and $y < 20 { echo "range" }"#);
}

#[test]
fn test_fix_with_parentheses() {
    let bad_code = r"if ($enabled) { if ($ready) { start } }";
    rule().assert_replacement_contains(bad_code, r"if ($enabled) and ($ready) { start }");
}
