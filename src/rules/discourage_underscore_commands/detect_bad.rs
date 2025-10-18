use super::rule;
use crate::LintContext;

#[test]
fn test_underscore_command_detected() {
    let bad_code = r"def my_command [param: string] {
    echo $param
}";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert!(
            !violations.is_empty(),
            "Should detect underscore in command name"
        );
        assert_eq!(violations[0].rule_id, "discourage_underscore_commands");
    });
}

#[test]
fn test_multiple_underscores_detected() {
    let bad_code = r"def my_very_long_command_name [param: string] {
    echo $param
}";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert!(
            !violations.is_empty(),
            "Should detect multiple underscores in command name"
        );
        assert_eq!(violations[0].rule_id, "discourage_underscore_commands");
    });
}
