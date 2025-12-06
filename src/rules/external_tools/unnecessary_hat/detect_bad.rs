use super::rule;

#[test]
fn detects_hat_on_simple_command() {
    rule().assert_detects("^ddcutil setvcp 10");
}

#[test]
fn detects_hat_on_command_no_args() {
    rule().assert_detects("^hostname");
}

#[test]
fn detects_hat_on_systemctl() {
    rule().assert_detects("^systemctl --user start timer.service");
}

#[test]
fn detects_hat_in_subexpression() {
    rule().assert_detects("let result = (^git status)");
}

#[test]
fn detects_hat_in_function() {
    rule().assert_detects(
        r#"
def run-command [] {
    ^curl https://example.com
}
"#,
    );
}

#[test]
fn detects_hat_with_flags() {
    rule().assert_detects("^git commit -m 'message'");
}
