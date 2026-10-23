use super::RULE;

#[test]
fn ignore_ansi_command() {
    RULE.assert_ignores(r#"print $"(ansi red)Error(ansi reset)""#);
}

#[test]
fn ignore_plain_string() {
    RULE.assert_ignores(r#"print "Hello World""#);
}

#[test]
fn ignore_string_without_escapes() {
    RULE.assert_ignores(r#"let msg = "No colors here""#);
}

#[test]
fn ignore_ansi_strip() {
    RULE.assert_ignores(r#"let clean = $text | ansi strip"#);
}

#[test]
fn ignore_ansi_gradient() {
    RULE.assert_ignores(r#"print ($text | ansi gradient --fgstart '0x40c9ff' --fgend '0xe81cff')"#);
}

#[test]
fn ignore_regular_escape_sequences() {
    RULE.assert_ignores(r#"print "Line 1\nLine 2\tTabbed""#);
}

#[test]
fn ignore_ansi_with_hex() {
    RULE.assert_ignores(r#"print $"(ansi '#FF0000')Red text(ansi reset)""#);
}

#[test]
fn ignore_ansi_escape_flag() {
    RULE.assert_ignores(r#"print $"(ansi --escape '31m')Red(ansi reset)""#);
}

#[test]
fn ignore_numbers_in_string() {
    RULE.assert_ignores(r#"let version = "v1.0[31]""#);
}
