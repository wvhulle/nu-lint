use super::rule;

#[test]
fn test_ignore_numeric_log_levels() {
    for (level, msg) in [
        ("0", "System emergency"),
        ("1", "Alert condition"),
        ("2", "Critical failure"),
        ("3", "Error occurred"),
        ("4", "Warning: deprecated feature"),
        ("5", "System notification"),
        ("6", "Starting process"),
        ("7", "Debug: entering function"),
    ] {
        rule().assert_ignores(&format!(r#"print "<{level}>{msg}""#));
    }
}

#[test]
fn test_ignore_keyword_log_levels() {
    for keyword in [
        "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
    ] {
        rule().assert_ignores(&format!(r#"print "<{keyword}>Test message""#));
    }
}

#[test]
fn test_ignore_mixed_levels() {
    let good_code = r#"
print "<6>Starting"
print "<4>Warning detected"
print "<3>Error occurred"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_keyword_and_numeric() {
    let good_code = r#"
print "<info>Starting"
print "<4>Warning detected"
print "<err>Error occurred"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_interpolated_strings_with_prefix() {
    let good_code = r#"print $"<6>Monitoring ($keyboard) for ($desc)""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_interpolated_strings_various_levels() {
    for (level, var) in [
        ("0", "emergency"),
        ("1", "alert"),
        ("2", "critical"),
        ("3", "error"),
        ("4", "warning"),
        ("5", "notice"),
        ("6", "info"),
        ("7", "debug"),
    ] {
        rule().assert_ignores(&format!(r#"print $"<{level}>Status: ($({var}))""#));
    }
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
    let good_code = r#"print $"<6>Task ($task_id) completed in ($duration)ms by ($user)""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_interpolated_strings_with_echo() {
    rule().assert_ignores(r#"echo $"<info>Server ($hostname) is running""#);
    rule().assert_ignores(r#"echo $"<6>Connection from ($ip_addr)""#);
}
