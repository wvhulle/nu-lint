use super::rule;

#[test]
fn test_fix_error_message() {
    let bad_code = r#"print "Error: failed to connect""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<err>");
    rule().assert_fix_explanation_contains(bad_code, "<err>");
}

#[test]
fn test_fix_warning_message() {
    let bad_code = r#"print "Warning: disk space low""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<warning>");
    rule().assert_fix_explanation_contains(bad_code, "<warning>");
}

#[test]
fn test_fix_info_message() {
    let bad_code = r#"print "Starting process""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<info>");
    rule().assert_fix_explanation_contains(bad_code, "<info>");
}

#[test]
fn test_fix_debug_message() {
    let bad_code = r#"print "Debug: entering function""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<debug>");
    rule().assert_fix_explanation_contains(bad_code, "<debug>");
}

#[test]
fn test_fix_critical_message() {
    let bad_code = r#"print "Critical: system failure""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<crit>");
    rule().assert_fix_explanation_contains(bad_code, "<crit>");
}

#[test]
fn test_fix_alert_message() {
    let bad_code = r#"print "Alert: immediate action required""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<alert>");
}

#[test]
fn test_fix_emergency_message() {
    let bad_code = r#"print "Emergency: system unusable""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<emerg>");
}

#[test]
fn test_fix_notice_message() {
    let bad_code = r#"print "Notice: configuration updated""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<notice>");
}

#[test]
fn test_fix_with_echo_command() {
    let bad_code = r#"echo "Error: connection timeout""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<err>");
    rule().assert_fix_explanation_contains(bad_code, "echo");
}

#[test]
fn test_fix_interpolated_string() {
    let bad_code = r#"print $"Error: failed to process ($file)""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<err>");
    rule().assert_replacement_contains(bad_code, "$\"");
}

#[test]
fn test_fix_single_quoted_string() {
    let bad_code = r"print 'Warning: low memory'";
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<warning>");
    rule().assert_replacement_contains(bad_code, "low memory");
}

#[test]
fn test_fix_detects_fail_keyword() {
    let bad_code = r#"print "Failed to load configuration""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<err>");
}

#[test]
fn test_fix_detects_warn_keyword() {
    let bad_code = r#"print "Warn: deprecated feature used""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<warning>");
}

#[test]
fn test_fix_plain_message_defaults_to_info() {
    let bad_code = r#"print "Starting application""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<info>");
}

#[test]
fn test_fix_in_function() {
    let bad_code = r#"
def deploy [] {
    print "Error: deployment failed"
}
"#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<err>");
}

#[test]
fn test_fix_multiple_violations() {
    let bad_code = r#"
print "Error: first issue"
print "Warning: second issue"
"#;
    rule().assert_count(bad_code, 2);
}

#[test]
fn test_fix_preserves_message_content() {
    let bad_code = r#"print "Error: connection to server failed""#;
    rule().assert_replacement_contains(bad_code, "connection to server failed");
}

#[test]
fn test_fix_explanation_shows_command() {
    let bad_code = r#"print "Error: timeout""#;
    rule().assert_fix_explanation_contains(bad_code, "print");
}

#[test]
fn test_fix_strips_error_prefix() {
    let bad_code = r#"print "Error: connection failed""#;
    rule().assert_replacement_contains(bad_code, "\"<err>connection failed\"");
}

#[test]
fn test_fix_strips_uppercase_error_prefix() {
    let bad_code = r#"print "ERROR: connection failed""#;
    rule().assert_replacement_contains(bad_code, "\"<err>connection failed\"");
}

#[test]
fn test_fix_strips_warning_prefix() {
    let bad_code = r#"print "Warning: disk space low""#;
    rule().assert_replacement_contains(bad_code, "\"<warning>disk space low\"");
}

#[test]
fn test_fix_strips_uppercase_warning_prefix() {
    let bad_code = r#"print "WARNING: disk space low""#;
    rule().assert_replacement_contains(bad_code, "\"<warning>disk space low\"");
}

#[test]
fn test_fix_strips_critical_prefix() {
    let bad_code = r#"print "Critical: system failure""#;
    rule().assert_replacement_contains(bad_code, "\"<crit>system failure\"");
}

#[test]
fn test_fix_strips_debug_prefix() {
    let bad_code = r#"print "Debug: entering function""#;
    rule().assert_replacement_contains(bad_code, "\"<debug>entering function\"");
}

#[test]
fn test_fix_strips_info_prefix() {
    let bad_code = r#"print "Info: process started""#;
    rule().assert_replacement_contains(bad_code, "\"<info>process started\"");
}

#[test]
fn test_fix_strips_err_prefix() {
    let bad_code = r#"print "err: something went wrong""#;
    rule().assert_replacement_contains(bad_code, "\"<err>something went wrong\"");
}

#[test]
fn test_fix_strips_warn_prefix() {
    let bad_code = r#"print "warn: be careful""#;
    rule().assert_replacement_contains(bad_code, "\"<warning>be careful\"");
}

#[test]
fn test_fix_strips_single_quoted_error() {
    let bad_code = r"print 'Error: failed operation'";
    rule().assert_replacement_contains(bad_code, "'<err>failed operation'");
}

#[test]
fn test_fix_strips_interpolated_error() {
    let bad_code = r#"print $"Error: ($details)""#;
    rule().assert_replacement_contains(bad_code, "$\"<err>($details)\"");
}

#[test]
fn test_fix_numeric_to_keyword_level_0() {
    let bad_code = r#"print "<0>System is unusable""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<emerg>");
    rule().assert_replacement_contains(bad_code, "System is unusable");
}

#[test]
fn test_fix_numeric_to_keyword_level_1() {
    let bad_code = r#"print "<1>Action must be taken immediately""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<alert>");
}

#[test]
fn test_fix_numeric_to_keyword_level_2() {
    let bad_code = r#"print "<2>Critical conditions""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<crit>");
}

#[test]
fn test_fix_numeric_to_keyword_level_3() {
    let bad_code = r#"print "<3>Error conditions""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<err>");
}

#[test]
fn test_fix_numeric_to_keyword_level_4() {
    let bad_code = r#"print "<4>Warning conditions""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<warning>");
}

#[test]
fn test_fix_numeric_to_keyword_level_5() {
    let bad_code = r#"print "<5>Normal but significant""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<notice>");
}

#[test]
fn test_fix_numeric_to_keyword_level_6() {
    let bad_code = r#"print "<6>Informational messages""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<info>");
}

#[test]
fn test_fix_numeric_to_keyword_level_7() {
    let bad_code = r#"print "<7>Debug-level messages""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<debug>");
}

#[test]
fn test_fix_numeric_prefix_with_echo() {
    let bad_code = r#"echo "<4>Low disk space""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<warning>");
    rule().assert_fix_explanation_contains(bad_code, "echo");
}

#[test]
fn test_fix_numeric_prefix_single_quoted() {
    let bad_code = r"print '<3>Connection failed'";
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<err>");
}

#[test]
fn test_fix_numeric_prefix_interpolated() {
    let bad_code = r#"print $"<6>Processing ($file)""#;
    rule().assert_detects(bad_code);
    rule().assert_replacement_contains(bad_code, "<info>");
    rule().assert_replacement_contains(bad_code, "($file)");
}
