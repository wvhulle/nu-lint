use super::rule;

#[test]
fn test_exit_code_checked() {
    let good_code = r"
let result = (^bluetoothctl info $mac | complete)
if $result.exit_code != 0 {
    return
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_no_complete_not_flagged() {
    let good_code = r"
let result = (some | regular | pipeline)
";
    rule().assert_ignores(good_code);
}
