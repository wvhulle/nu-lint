use super::rule;

#[test]
fn test_ignore_with_complete_between() {
    let good_code = r"
let result = (^command1 | complete)
if $result.exit_code == 0 {
    ^command2
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_with_conditional() {
    let good_code = r"
^command1 && ^command2
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_single_external() {
    let good_code = r"
^command1
";
    rule().assert_ignores(good_code);
}
