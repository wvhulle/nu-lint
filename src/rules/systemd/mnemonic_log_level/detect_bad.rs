use super::rule;

#[test]
fn test_detect_numeric_prefix() {
    let bad_code = r#"print "<6>Starting service""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_all_numeric_prefixes() {
    for level in 0..=7 {
        let bad_code = format!(r#"print "<{level}>Test message""#);
        rule().assert_detects(&bad_code);
    }
}

#[test]
fn test_detect_numeric_prefix_in_function() {
    let bad_code = r#"
def main [] {
    print "<3>Error occurred"
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_numeric_prefix_with_echo() {
    let bad_code = r#"echo "<4>Warning message""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_numeric_prefix_interpolated() {
    let bad_code = r#"print $"<5>Notice: ($msg)""#;
    rule().assert_detects(bad_code);
}
