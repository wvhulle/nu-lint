use super::RULE;

#[test]
fn ignore_single_str_replace() {
    RULE.assert_ignores("$text | str replace 'a' 'b'");
}

#[test]
fn ignore_with_other_commands_between() {
    // Not consecutive, so don't flag
    RULE.assert_ignores("$text | str replace 'a' 'b' | str trim | str replace 'c' 'd'");
}

#[test]
fn ignore_str_replace_with_other_commands() {
    RULE.assert_ignores("$text | str replace 'a' 'b' | str downcase");
}

#[test]
fn ignore_different_string_operations() {
    RULE.assert_ignores("$text | str replace 'a' 'b' | str trim");
}

#[test]
fn ignore_no_str_replace() {
    RULE.assert_ignores("$text | str trim | str downcase");
}
