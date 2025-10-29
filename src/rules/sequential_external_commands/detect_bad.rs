use super::rule;

#[test]
fn test_detect_sequential_without_check() {
    let bad_code = r"
^git add .
^git commit -m 'message'
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sequential_on_same_line() {
    let bad_code = r"^command1; ^command2";
    rule().assert_detects(bad_code);
}
