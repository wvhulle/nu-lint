use super::rule;

#[test]
fn detects_comma_in_list() {
    let code = "let items = [1, 2, 3]";

    rule().assert_detects(code);
    rule().assert_violation_count_exact(code, 2);
}

#[test]
fn detects_multiple_commas_in_list() {
    let code = r#"let fruits = ["apple", "banana", "cherry"]"#;

    rule().assert_detects(code);
    rule().assert_violation_count_exact(code, 2);
}

#[test]
fn detects_commas_in_nested_list() {
    let code = "let matrix = [[1, 2], [3, 4]]";

    rule().assert_violation_count_exact(code, 3);
}
