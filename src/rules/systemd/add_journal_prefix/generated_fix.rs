use super::RULE;

#[test]
fn test_fix_all_log_levels() {
    let cases = [
        (r#"print "Error: failed""#, "<3>"),
        (r#"print "ERROR: failed""#, "<3>"),
        (r#"print "err: failed""#, "<3>"),
        (r#"print "Failed to load""#, "<3>"),
        (r#"print "Warning: low""#, "<4>"),
        (r#"print "WARNING: low""#, "<4>"),
        (r#"print "warn: low""#, "<4>"),
        (r#"print "Critical: failure""#, "<2>"),
        (r#"print "Debug: entering""#, "<7>"),
        (r#"print "Info: started""#, "<6>"),
        (r#"print "Alert: action required""#, "<1>"),
        (r#"print "Emergency: unusable""#, "<0>"),
        (r#"print "Notice: updated""#, "<5>"),
        (r#"print "Starting process""#, "<6>"), // default
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
        "\"<3>connection failed\"",
    );
    RULE.assert_replacement_contains(r#"print "Warning: disk low""#, "\"<4>disk low\"");
    RULE.assert_replacement_contains(
        r#"print "Debug: entering function""#,
        "\"<7>entering function\"",
    );
}

#[test]
fn test_fix_echo_command() {
    let bad_code = r#"echo "Error: timeout""#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "<3>");
}

#[test]
fn test_fix_single_quoted_string() {
    let bad_code = r"print 'Error: failed operation'";
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "'<3>failed operation'");
}

#[test]
fn test_fix_interpolated_string() {
    let bad_code = r#"print $"Error: ($details)""#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "$\"<3>($details)\"");
}

#[test]
fn test_fix_in_function() {
    let bad_code = r#"
def deploy [] {
    print "Error: deployment failed"
}
"#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "<3>");
}

#[test]
fn test_fix_isolated_print() {
    let bad_code = r#"
let x = 1
print "Error: first issue"
let y = 2
"#;
    RULE.assert_replacement_contains(bad_code, "<3>");
}
