use super::RULE;

#[test]
fn test_two_env_assignments() {
    RULE.assert_detects(
        r#"
$env.VAR1 = "value1"
$env.VAR2 = "value2"
"#,
    );
}

#[test]
fn test_three_env_assignments() {
    RULE.assert_detects(
        r#"
$env.PATH = "/usr/bin"
$env.HOME = "/home/user"
$env.SHELL = "/bin/nu"
"#,
    );
}

#[test]
fn test_in_closure() {
    RULE.assert_detects(
        r#"
def setup [] {
    $env.VAR1 = "a"
    $env.VAR2 = "b"
}
"#,
    );
}
