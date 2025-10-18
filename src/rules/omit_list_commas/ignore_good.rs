use super::rule;
use crate::LintContext;

#[test]
fn ignores_list_without_commas() {
    let code = "let items = [1 2 3]";

    LintContext::test_with_parsed_source(code, |context| {
        assert!((rule().check)(&context).is_empty());
    });
}

#[test]
fn ignores_empty_list() {
    let code = "let empty = []";

    LintContext::test_with_parsed_source(code, |context| {
        assert!((rule().check)(&context).is_empty());
    });
}

#[test]
fn ignores_single_item_list() {
    let code = "let single = [42]";

    LintContext::test_with_parsed_source(code, |context| {
        assert!((rule().check)(&context).is_empty());
    });
}

#[test]
fn ignores_multiline_list_without_commas() {
    let code = r#"let items = [
    "first"
    "second"
    "third"
]"#;

    LintContext::test_with_parsed_source(code, |context| {
        assert!((rule().check)(&context).is_empty());
    });
}
