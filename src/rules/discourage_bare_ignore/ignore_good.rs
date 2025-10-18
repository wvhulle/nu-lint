use super::rule;
use crate::LintContext;

#[test]
fn test_external_command_ignore_acceptable() {

    let acceptable_code = r"
^bluetoothctl power on | ignore
";
    LintContext::test_with_parsed_source(acceptable_code, |context| {
        assert_eq!(
            (rule().check)(&context).len(),
            0,
            "Should not flag external command ignore"
        );
    });
}

#[test]
fn test_do_ignore_not_flagged() {

    let good_code = r"
do -i { some | pipeline }
";
    LintContext::test_with_parsed_source(good_code, |context| {
        assert_eq!((rule().check)(&context).len(), 0, "Should not flag do -i");
    });
}
