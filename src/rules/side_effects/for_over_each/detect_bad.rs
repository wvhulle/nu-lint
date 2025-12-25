use super::RULE;

#[test]
fn test_detect_each_with_print() {
    let bad_code = r"[1 2 3] | each {|x| print $x}";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_each_with_multiple_prints() {
    let bad_code = r#"
[1 2 3] | each {|x|
    print "Value:"
    print $x
}
"#;
    RULE.assert_detects(bad_code);
}
