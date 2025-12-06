use super::rule;

#[test]
fn fixes_hat_with_multiple_args() {
    rule().assert_replacement_contains("^ddcutil setvcp 10 90", "ddcutil setvcp 10 90");
}

#[test]
fn fixes_hat_no_args() {
    rule().assert_replacement_contains("^hostname", "hostname");
}

#[test]
fn fixes_hat_with_flags() {
    rule().assert_replacement_contains("^git status --short", "git status --short");
}

#[test]
fn fixes_hat_preserves_quoted_args() {
    rule().assert_replacement_contains("^git commit -m 'message'", "git commit -m 'message'");
}

#[test]
fn fix_explanation_mentions_remove() {
    rule().assert_fix_explanation_contains("^hostname", "Remove");
}
