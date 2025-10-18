use super::rule;
use crate::LintContext;

#[test]
fn test_exported_function_with_docs() {
    let source = r#"
# This is a documented command
export def my-command [] {
    echo "hello"
}
"#;
    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);

        assert!(
            violations.is_empty(),
            "Should not flag documented exported functions"
        );
    });
}

#[test]
fn test_non_exported_function_without_docs() {
    let source = r#"
def my-command [] {
    echo "hello"
}
"#;
    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);

        assert!(
            violations.is_empty(),
            "Should not flag non-exported functions"
        );
    });
}

#[test]
fn test_exported_function_with_multi_line_docs() {
    let source = r"
# Process input data
# Returns the processed result
export def process-data [input: string] {
    echo $input
}
";
    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);

        assert!(
            violations.is_empty(),
            "Should not flag exported functions with documentation"
        );
    });
}
