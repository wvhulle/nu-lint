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
fn test_ignore_with_echo_command() {
    rule().assert_ignores(r#"echo "<5>System notification""#);
    rule().assert_ignores(r#"echo "<info>Starting process""#);
}

#[test]
fn test_ignore_multiple_prints_with_prefixes() {
    let good_code = r#"
print "<6>Starting task"
print "<6>Task completed"
"#;
    rule().assert_ignores(good_code);
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
