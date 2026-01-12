use super::RULE;

#[test]
fn detects_comma_in_list() {
    let code = "let items = [1, 2, 3]";

    RULE.assert_detects(code);
    RULE.assert_count(code, 2);
}

#[test]
fn detects_multiple_commas_in_list() {
    let code = r#"let fruits = ["apple", "banana", "cherry"]"#;

    RULE.assert_detects(code);
    RULE.assert_count(code, 2);
}

#[test]
fn detects_commas_in_nested_list() {
    let code = "let matrix = [[1, 2], [3, 4]]";

    RULE.assert_count(code, 3);
}

#[test]
fn detects_comma_before_comment_with_comma() {
    let code = r#"let responses = [
    "item1", # comment with comma 2,3
    "item2"
]"#;
    // Should detect 1 comma: after "item1" (the comma in the comment should be
    // ignored)
    RULE.assert_count(code, 1);
}
