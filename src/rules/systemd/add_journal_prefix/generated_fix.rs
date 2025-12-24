use super::RULE;

#[test]
fn test_fix_all_log_levels() {
    let cases = [
        (r#"print "Error: failed""#, "<err>"),
        (r#"print "ERROR: failed""#, "<err>"),
        (r#"print "err: failed""#, "<err>"),
        (r#"print "Failed to load""#, "<err>"),
        (r#"print "Warning: low""#, "<warning>"),
        (r#"print "WARNING: low""#, "<warning>"),
        (r#"print "warn: low""#, "<warning>"),
        (r#"print "Critical: failure""#, "<crit>"),
        (r#"print "Debug: entering""#, "<debug>"),
        (r#"print "Info: started""#, "<info>"),
        (r#"print "Alert: action required""#, "<alert>"),
        (r#"print "Emergency: unusable""#, "<emerg>"),
        (r#"print "Notice: updated""#, "<notice>"),
        (r#"print "Starting process""#, "<info>"), // default
    ];
    for (code, expected_prefix) in cases {
        RULE.assert_detects(code);
        RULE.assert_replacement_contains(code, expected_prefix);
    }
}

#[test]
fn test_fix_strips_redundant_prefix() {
    RULE.assert_replacement_contains(
        r#"print "Error: connection failed""#,
        "\"<err>connection failed\"",
    );
    RULE.assert_replacement_contains(r#"print "Warning: disk low""#, "\"<warning>disk low\"");
    RULE.assert_replacement_contains(
        r#"print "Debug: entering function""#,
        "\"<debug>entering function\"",
    );
}

#[test]
fn test_fix_echo_command() {
    let bad_code = r#"echo "Error: timeout""#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "<err>");
}

#[test]
fn test_fix_single_quoted_string() {
    let bad_code = r"print 'Error: failed operation'";
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "'<err>failed operation'");
}

#[test]
fn test_fix_interpolated_string() {
    let bad_code = r#"print $"Error: ($details)""#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "$\"<err>($details)\"");
}

#[test]
fn test_fix_in_function() {
    let bad_code = r#"
def deploy [] {
    print "Error: deployment failed"
}
"#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "<err>");
}

#[test]
fn test_fix_isolated_print() {
    let bad_code = r#"
let x = 1
print "Error: first issue"
let y = 2
"#;
    RULE.assert_replacement_contains(bad_code, "<err>");
}
