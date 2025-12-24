use super::RULE;

#[test]
fn test_ignore_keyword_log_levels() {
    for keyword in [
        "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
    ] {
        RULE.assert_ignores(&format!(r#"print "<{keyword}>Test message""#));
    }
}

#[test]
fn test_ignore_numeric_prefixes() {
    for level in 0..=7 {
        RULE.assert_ignores(&format!(r#"print "<{level}>Test message""#));
    }
}

#[test]
fn test_ignore_mixed_keyword_levels() {
    let good_code = r#"
print "<info>Starting"
print "<warning>Warning detected"
print "<err>Error occurred"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_multiline_strings() {
    let good_code = r#"
print "Usage: script.nu <subcommand>

Subcommands:
  help - Show help
  version - Show version"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_interpolated_strings_with_keyword_prefix() {
    let good_code = r#"print $"<info>Monitoring ($keyboard) for ($desc)""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_consecutive_prints() {
    let good_code = r#"
print "Usage: script.nu <subcommand>"
print ""
print "Subcommands:"
print "  help - Show help"
print "  version - Show version"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_consecutive_prints_in_function() {
    let good_code = r#"
def show_help [] {
    print "Available commands:"
    print "  start - Start the service"
    print "  stop - Stop the service"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_print_followed_by_piped_print() {
    let good_code = r#"
def list_items [] {
    print "Available items:"
    get_items | print
}
"#;
    RULE.assert_ignores(good_code);
}
