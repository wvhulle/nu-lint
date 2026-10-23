use super::RULE;

#[test]
fn test_fix_split_first_space() {
    let bad_code = r#""hello world foo" | split row " " | first"#;
    let expected = r#""hello world foo" | split words | first"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_split_last_space() {
    let bad_code = r#""hello world foo" | split row " " | last"#;
    let expected = r#""hello world foo" | split words | last"#;
    RULE.assert_fixed_is(bad_code, expected);
}
