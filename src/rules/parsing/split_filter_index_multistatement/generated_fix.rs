use super::RULE;

#[test]
fn test_fix_simple_split_get() {
    let bad_code = r#"
let split = ("a:b:c" | split row ":")
$split | get 0
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0}:{field1}:{field2}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field0");
}

#[test]
fn test_fix_split_get_different_index() {
    let bad_code = r#"
let parts = ("hello world" | split row " ")
$parts | get 1
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0} {field1}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field1");
}

#[test]
fn test_fix_with_filter() {
    let bad_code = r#"
let split = ("a,b,c" | split row "," | filter {|x| $x != "b"})
$split | get 0
"#;
    // Note: This fix won't preserve filter semantics perfectly, but suggests parse
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0},{field1},{field2}""#);
    RULE.assert_fixed_contains(bad_code, "get 0.field0");
}
