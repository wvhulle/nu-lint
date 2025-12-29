use super::RULE;

#[test]
fn test_detect_each_with_split_row() {
    let bad_code = r#"
$data | lines | each { |line| $line | split row " " }
"#;

    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_each_with_split() {
    let bad_code = r#"
$lines | each { |l| $l | split " " }
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_nested_split_in_each() {
    let bad_code = r#"
$text | lines | each { |line|
    let parts = ($line | split row ":")
    $parts
}
"#;

    RULE.assert_detects(bad_code);
}
