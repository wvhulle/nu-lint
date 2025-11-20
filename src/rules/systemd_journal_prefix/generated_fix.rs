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
    rule().assert_replacement_contains(bad_code, "Warning: low memory");
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
