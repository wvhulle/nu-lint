use super::RULE;

#[test]
fn test_fix_simple_each_with_print() {
    let bad_code = r"[1 2 3] | each {|x| print $x}";
    let expected = r"for x in [1 2 3] { print $x }";
    RULE.assert_fixed_contains(bad_code, expected);
}

#[test]
fn test_fix_each_with_multiple_prints() {
    let bad_code = r#"[1 2 3] | each {|x|
    print "Value:"
    print $x
}"#;
    let expected = r#"for x in [1 2 3] { print "Value:"
    print $x }"#;
    RULE.assert_fixed_contains(bad_code, expected);
}

// Complex pipelines with multiple stages before `each` don't get auto-fixes.
// Detection is still tested in detect_bad.rs. Users should manually decide
// whether to restructure the code or add `| ignore`.
