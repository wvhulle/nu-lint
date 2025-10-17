use super::*;

#[test]
fn test_external_command_ignore_acceptable() {
    let rule = DiscouragedBareIgnore::new();

    let acceptable_code = r"
^bluetoothctl power on | ignore
";
    let context = LintContext::test_from_source(acceptable_code);
    assert_eq!(
        rule.check(&context).len(),
        0,
        "Should not flag external command ignore"
    );
}

#[test]
fn test_do_ignore_not_flagged() {
    let rule = DiscouragedBareIgnore::new();

    let good_code = r"
do -i { some | pipeline }
";
    let context = LintContext::test_from_source(good_code);
    assert_eq!(rule.check(&context).len(), 0, "Should not flag do -i");
}