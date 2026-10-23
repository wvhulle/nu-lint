use super::RULE;

#[test]
fn ignore_get_with_optional_flag() {
    RULE.assert_ignores("$list | get -o 0");
}

#[test]
fn ignore_get_with_long_optional_flag() {
    RULE.assert_ignores("$list | get --optional 0");
}

#[test]
fn ignore_inside_try_block() {
    RULE.assert_ignores("try { $list | get 0 }");
}

#[test]
fn ignore_get_with_string_key() {
    // String keys are for records, not list access
    RULE.assert_ignores("$record | get name");
}

#[test]
fn ignore_get_with_variable_key() {
    // Variable keys might be strings (handled by unsafe_dynamic_record_access rule)
    RULE.assert_ignores("$record | get $key");
}
