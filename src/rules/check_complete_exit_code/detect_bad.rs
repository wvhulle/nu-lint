use super::rule;

#[test]
fn test_missing_exit_code_check() {
    let bad_code = r"
let result = (^bluetoothctl info $mac | complete)
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_mut_missing_exit_code_check() {
    let bad_code = r"
mut result = (^git status | complete)
";

    rule().assert_detects(bad_code);
}
