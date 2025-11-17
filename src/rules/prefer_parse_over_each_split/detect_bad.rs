use super::rule;

#[test]
fn test_detect_each_with_split_row() {
    let bad_code = r#"
$data | lines | each { |line| $line | split row " " }
"#;

    rule().assert_count(bad_code, 1);
}

#[test]
fn test_detect_each_with_split() {
    let bad_code = r#"
$lines | each { |l| $l | split " " }
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_nested_split_in_each() {
    let bad_code = r#"
$text | lines | each { |line|
    let parts = ($line | split row ":")
    $parts
}
"#;

    rule().assert_detects(bad_code);
}
