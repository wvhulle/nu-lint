use super::RULE;

#[test]
fn detects_hat_on_simple_command() {
    RULE.assert_detects("ddcutil setvcp 10");
}

#[test]
fn detects_hat_on_command_no_args() {
    RULE.assert_detects("hostname");
}

#[test]
fn detects_hat_on_systemctl() {
    RULE.assert_detects("systemctl --user start timer.service");
}

#[test]
fn detects_hat_in_subexpression() {
    RULE.assert_detects("let result = (git status)");
}

#[test]
fn detects_hat_in_function() {
    RULE.assert_detects(
        r#"
def run-command [] {
    curl https://example.com
}
"#,
    );
}

#[test]
fn detects_hat_with_flags() {
    RULE.assert_detects("git commit -m 'message'");
}
