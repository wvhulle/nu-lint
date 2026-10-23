use super::RULE;

#[test]
fn ignore_long_flag_all() {
    RULE.assert_ignores("ls --all");
}

#[test]
fn ignore_long_flag_long() {
    RULE.assert_ignores("ls --long");
}

#[test]
fn ignore_long_flag_with_value() {
    RULE.assert_ignores("str replace --regex 'a' 'b'");
}

#[test]
fn ignore_no_flags() {
    RULE.assert_ignores("ls");
}

#[test]
fn ignore_positional_only() {
    RULE.assert_ignores("ls /tmp");
}

#[test]
fn ignore_custom_commands() {
    // Custom commands should not trigger this rule
    RULE.assert_ignores(
        r#"
        def my-cmd [-f] { print $f }
        my-cmd -f
    "#,
    );
}

#[test]
fn ignore_mixed_with_long() {
    // When long flags are used, they shouldn't trigger
    RULE.assert_ignores("ls --all /tmp");
}
