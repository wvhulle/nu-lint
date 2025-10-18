use super::rule;
use crate::LintContext;

#[test]
fn detects_comma_in_list() {
    let code = "let items = [1, 2, 3]";

    LintContext::test_with_parsed_source(code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}

#[test]
fn detects_multiple_commas_in_list() {
    let code = r#"let fruits = ["apple", "banana", "cherry"]"#;

    LintContext::test_with_parsed_source(code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}

#[test]
fn detects_commas_in_nested_list() {
    let code = "let matrix = [[1, 2], [3, 4]]";

    LintContext::test_with_parsed_source(code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}
