#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::exported_function_docs::ExportedFunctionDocs;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_exported_function_with_docs() {
        let source = r#"
# This is a documented command
export def my-command [] {
    echo "hello"
}
"#;
        let rule = ExportedFunctionDocs::new();
        let context = LintContext::test_from_source(source);
        let violations = rule.check(&context);

        assert!(
            violations.is_empty(),
            "Should not flag documented exported functions"
        );
    }

    #[test]
    fn test_non_exported_function_without_docs() {
        let source = r#"
def my-command [] {
    echo "hello"
}
"#;
        let rule = ExportedFunctionDocs::new();
        let context = LintContext::test_from_source(source);
        let violations = rule.check(&context);

        assert!(
            violations.is_empty(),
            "Should not flag non-exported functions"
        );
    }

    #[test]
    fn test_exported_function_with_multi_line_docs() {
        let source = r#"
# Process input data
# Returns the processed result
export def process-data [input: string] {
    echo $input
}
"#;
        let rule = ExportedFunctionDocs::new();
        let context = LintContext::test_from_source(source);
        let violations = rule.check(&context);

        assert!(
            violations.is_empty(),
            "Should not flag exported functions with documentation"
        );
    }
}
