use super::RULE;

#[test]
fn test_fix_numeric_to_keyword_all_levels() {
    let cases = [
        (r#"print "<0>System unusable""#, "<emerg>"),
        (r#"print "<1>Action required""#, "<alert>"),
        (r#"print "<2>Critical conditions""#, "<crit>"),
        (r#"print "<3>Error conditions""#, "<err>"),
        (r#"print "<4>Warning conditions""#, "<warning>"),
        (r#"print "<5>Normal significant""#, "<notice>"),
        (r#"print "<6>Informational""#, "<info>"),
        (r#"print "<7>Debug messages""#, "<debug>"),
    ];
    for (code, expected_keyword) in cases {
        RULE.assert_detects(code);
        RULE.assert_replacement_contains(code, expected_keyword);
    }
}

#[test]
fn test_fix_numeric_prefix_preserves_string_type() {
    RULE.assert_replacement_contains(r"print '<3>Connection failed'", "'<err>");
    RULE.assert_replacement_contains(r#"print $"<6>Processing ($file)""#, "$\"<info>");
}

#[test]
fn test_fix_echo_command() {
    let bad_code = r#"echo "<4>Warning: check logs""#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "<warning>");
}

#[test]
fn test_fix_in_function() {
    let bad_code = r#"
def main [] {
    print "<3>Error: something went wrong"
}
"#;
    RULE.assert_detects(bad_code);
    RULE.assert_replacement_contains(bad_code, "<err>");
}
