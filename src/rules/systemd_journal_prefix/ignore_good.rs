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
fn test_ignore_mixed_keyword_levels() {
    let good_code = r#"
print "<info>Starting"
print "<warning>Warning detected"
print "<err>Error occurred"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_interpolated_strings_with_keyword_prefix() {
    let good_code = r#"print $"<info>Monitoring ($keyboard) for ($desc)""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_interpolated_strings_keyword_levels() {
    for keyword in [
        "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
    ] {
        rule().assert_ignores(&format!(r#"print $"<{keyword}>Process ($name) started""#));
    }
}

#[test]
fn test_ignore_interpolated_strings_multiple_vars() {
    let good_code = r#"print $"<info>Task ($task_id) completed in ($duration)ms by ($user)""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_interpolated_strings_with_echo() {
    rule().assert_ignores(r#"echo $"<info>Server ($hostname) is running""#);
    rule().assert_ignores(r#"echo $"<warning>Connection from ($ip_addr)""#);
}
