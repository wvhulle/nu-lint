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
