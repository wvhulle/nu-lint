use super::rule;

#[test]
fn test_detect_split_row_with_newline_double_quotes() {
    let bad_code = r#"
$text | split row "\n"
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_with_newline_single_quotes() {
    let bad_code = r"
$text | split row '\n'
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_split_row_multiline() {
    let bad_code = r#"
def process-text [] {
    $input | split row "\n" | each { |line| $line | str trim }
}
"#;

    rule().assert_detects(bad_code);
}
