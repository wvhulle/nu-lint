use super::RULE;

#[test]
fn test_fix_split_row_first_colon() {
    let bad_code = r#""a:b:c" | split row ":" | first"#;
    let expected = r#""a:b:c" | parse "{first}:{_}" | get first"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_split_row_first_slash() {
    let bad_code = r#""path/to/file" | split row "/" | first"#;
    let expected = r#""path/to/file" | parse "{first}/{_}" | get first"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_fix_split_row_first_comma() {
    let bad_code = r#""a,b,c" | split row "," | first"#;
    let expected = r#""a,b,c" | parse "{first},{_}" | get first"#;
    RULE.assert_fixed_is(bad_code, expected);
}
