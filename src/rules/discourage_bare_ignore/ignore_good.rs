use super::rule;

#[test]
fn test_external_command_ignore_acceptable() {
    let acceptable_code = r"
^bluetoothctl power on | ignore
";
    rule().assert_ignores(acceptable_code);
}

#[test]
fn test_do_ignore_not_flagged() {
    let good_code = r"
do -i { some | pipeline }
";
    rule().assert_ignores(good_code);
}
