use super::rule;

#[test]
fn test_underscore_command_detected() {
    let bad_code = r"def my_command [param: string] {
    echo $param
}";
    rule().assert_violation_count(bad_code, 1);
}

#[test]
fn test_multiple_underscores_detected() {
    let bad_code = r"def my_very_long_command_name [param: string] {
    echo $param
}";
    rule().assert_violation_count(bad_code, 1);
}
