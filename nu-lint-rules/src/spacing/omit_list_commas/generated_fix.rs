use super::RULE;

#[test]
fn detects_violations_for_comma_in_list() {
    let code = "let items = [1, 2, 3]";
    RULE.assert_detects(code);
}

#[test]
fn fixes_comma_preserving_comment() {
    let bad_code = r#"let items = [
    "item1", # comment with comma 2,3
    "item2"
]"#;
    let expected = r#"let items = [
    "item1" # comment with comma 2,3
    "item2"
]"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn fixes_comma_with_inline_comment_after() {
    let bad_code = r#"let x = [1, # trailing,comment
2]"#;
    let expected = r#"let x = [1 # trailing,comment
2]"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn fixes_simple_comma() {
    let bad_code = "let x = [1, 2]";
    let expected = "let x = [1 2]";
    RULE.assert_fixed_is(bad_code, expected);
}
