use super::RULE;

#[test]
fn test_fix_split_first_colon() {
    let bad_code = r#""a:b:c" | split row ":" | first"#;
    RULE.assert_replacement_contains(bad_code, "parse --regex");
    RULE.assert_replacement_contains(bad_code, "first");
}

#[test]
fn test_fix_split_last_colon() {
    let bad_code = r#""a:b:c" | split row ":" | last"#;
    RULE.assert_replacement_contains(bad_code, "parse --regex");
    RULE.assert_replacement_contains(bad_code, "last");
}

#[test]
fn test_fix_split_first_slash() {
    let bad_code = r#""path/to/file" | split row "/" | first"#;
    RULE.assert_replacement_contains(bad_code, "parse --regex");
}

#[test]
fn test_fix_split_last_slash() {
    let bad_code = r#""path/to/file.txt" | split row "/" | last"#;
    RULE.assert_replacement_contains(bad_code, "parse --regex");
    RULE.assert_replacement_contains(bad_code, "last");
}
