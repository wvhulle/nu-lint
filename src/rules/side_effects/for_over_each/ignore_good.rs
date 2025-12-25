use super::RULE;

#[test]
fn test_for_loop_not_flagged() {
    let good_code = "for x in [1 2 3] { print $x }";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_each_with_echo_returns_data() {
    let good_code = "[1 2 3] | each {|x| echo $x}";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_each_returning_values() {
    let good_code = "[1 2 3] | each {|x| $x * 2}";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_each_with_transformation() {
    let good_code = "[1 2 3] | each {|x| $x | into string}";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_each_with_mixed_operations() {
    let good_code = r#"
[1 2 3] | each {|x|
    print $x
    $x * 2
}
"#;
    RULE.assert_ignores(good_code);
}
