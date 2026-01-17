use super::RULE;

#[test]
fn fix_short_flag_to_long() {
    RULE.assert_fixed_is("ls -a", "ls --all");
}

#[test]
fn fix_short_flag_ls_long() {
    RULE.assert_fixed_is("ls -l", "ls --long");
}

#[test]
fn fix_short_flag_with_value() {
    RULE.assert_fixed_contains("str replace -r 'a' 'b'", "--regex");
}
