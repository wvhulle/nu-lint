use super::RULE;

#[test]
fn test_fix_two_env() {
    RULE.assert_fixed_is(
        r#"$env.VAR1 = "value1"
$env.VAR2 = "value2""#,
        r#"load-env { VAR1: "value1", VAR2: "value2" }"#,
    );
}

#[test]
fn test_fix_three_env() {
    RULE.assert_fixed_is(
        r#"$env.A = 1
$env.B = 2
$env.C = 3"#,
        r#"load-env { A: 1, B: 2, C: 3 }"#,
    );
}

#[test]
fn test_fix_mixed_value_types() {
    RULE.assert_fixed_is(
        r#"$env.DEBUG = true
$env.PORT = 8080
$env.NAME = "app""#,
        r#"load-env { DEBUG: true, PORT: 8080, NAME: "app" }"#,
    );
}

#[test]
fn test_fix_with_expressions() {
    RULE.assert_fixed_is(
        r#"$env.PATH = ($env.PATH | prepend "/usr/local/bin")
$env.HOME = $nu.home-path"#,
        r#"load-env { PATH: ($env.PATH | prepend "/usr/local/bin"), HOME: $nu.home-path }"#,
    );
}

#[test]
fn test_fix_quoted_field_names() {
    RULE.assert_fixed_is(
        r#"$env.MY_VAR = "a"
$env.OTHER_VAR = "b""#,
        r#"load-env { MY_VAR: "a", OTHER_VAR: "b" }"#,
    );
}
