use super::rule;

#[test]
fn test_hyphenated_command_not_flagged() {
    let good_code = r"def my-command [param: string] {
    echo $param
}";
    rule().assert_ignores(good_code);
}

#[test]
fn test_single_word_command_not_flagged() {
    let good_code = r"def command [param: string] {
    echo $param
}";
    rule().assert_ignores(good_code);
}
