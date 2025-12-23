use super::rule;

#[test]
fn test_ignore_keyword_log_levels() {
    for keyword in [
        "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
    ] {
        rule().assert_ignores(&format!(r#"print "<{keyword}>Test message""#));
    }
}

#[test]
fn test_ignore_missing_prefix() {
    rule().assert_ignores(r#"print "Hello, World!""#);
    rule().assert_ignores(r#"echo "Starting process""#);
}

#[test]
fn test_ignore_multiline_strings() {
    let good_code = r#"
print "Usage: script.nu <subcommand>

Subcommands:
  help - Show help
  version - Show version"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_consecutive_prints_with_numeric() {
    let good_code = r#"
print "<6>Item 1"
print "<6>Item 2"
print "<6>Item 3"
"#;
    rule().assert_ignores(good_code);
}
