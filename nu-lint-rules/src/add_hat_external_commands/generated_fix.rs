use super::RULE;

#[test]
fn fixes_hat_with_multiple_args() {
    RULE.assert_fixed_contains("ddcutil setvcp 10 90", "^ddcutil setvcp 10 90");
}

#[test]
fn fixes_hat_no_args() {
    RULE.assert_fixed_contains("hostname", "^hostname");
}

#[test]
fn fixes_hat_with_flags() {
    RULE.assert_fixed_contains("git status --short", "^git status --short");
}

#[test]
fn fixes_hat_preserves_quoted_args() {
    RULE.assert_fixed_contains("git commit -m 'message'", "^git commit -m 'message'");
}
