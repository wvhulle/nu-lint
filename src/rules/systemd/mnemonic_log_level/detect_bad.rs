use super::RULE;

#[test]
fn test_detect_numeric_prefix() {
    let bad_code = r#"print "<6>Starting service""#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_all_numeric_prefixes() {
    for level in 0..=7 {
        let bad_code = format!(r#"print "<{level}>Test message""#);
        RULE.assert_detects(&bad_code);
    }
}

#[test]
fn test_detect_numeric_prefix_in_function() {
    let bad_code = r#"
def main [] {
    print "<3>Error occurred"
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_numeric_prefix_with_echo() {
    let bad_code = r#"echo "<4>Warning message""#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_numeric_prefix_interpolated() {
    let bad_code = r#"print $"<5>Notice: ($msg)""#;
    RULE.assert_detects(bad_code);
}
