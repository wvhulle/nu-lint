use super::RULE;

// Fix generation tests will be added in Phase 3
// Currently the fix() method returns None

#[test]
#[ignore = "Fix generation not yet implemented"]
fn test_fix_simple_split_get() {
    let bad_code = r#"
let split = ("a:b:c" | split row ":")
$split | get 0
"#;
    RULE.assert_fixed_contains(bad_code, r#"parse "{field0}:{field1}:{field2}""#);
}
