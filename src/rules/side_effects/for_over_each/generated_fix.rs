use super::RULE;

#[test]
fn test_fix_simple_each_with_print() {
    let bad_code = r"[1 2 3] | each {|x| print $x}";
    let expected = r"for x in [1 2 3] {|x| print $x}";
    RULE.assert_replacement_contains(bad_code, expected);
}

#[test]
fn test_fix_each_with_multiple_prints() {
    let bad_code = r#"[1 2 3] | each {|x|
    print "Value:"
    print $x
}"#;
    let expected = r#"for x in [1 2 3] {|x|
    print "Value:"
    print $x
}"#;
    RULE.assert_replacement_contains(bad_code, expected);
}
