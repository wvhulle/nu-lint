use super::RULE;

#[test]
fn test_get_ignore_errors_suggests_optional_flag() {
    let code = "{a: 1} | get --ignore-errors b";
    RULE.assert_help_contains(code, "--optional (-o)");
}

#[test]
fn test_get_ignore_errors_explains_rename_reason() {
    let code = "{a: 1} | get --ignore-errors b";
    RULE.assert_help_contains(code, "to better reflect its behavior");
}

#[test]
fn test_get_ignore_errors_mentions_flag_renamed() {
    let code = "{a: 1} | get --ignore-errors b";
    RULE.assert_help_contains(code, "has been renamed");
}
