use super::RULE;

#[test]
fn test_detect_split_row_with_newline() {
    let bad_code = r#"
$text | split row "\n"
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_multiline() {
    let bad_code = r#"
def process-text [] {
    $input | split row "\n" | each { |line| $line | str trim }
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn detect_newline_written_as_unicode_escape() {
    RULE.assert_detects(r#"$text | split row "\u{a}""#);
}
