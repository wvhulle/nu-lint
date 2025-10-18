use super::rule;
use crate::LintContext;

#[test]
fn test_exported_function_without_docs() {
    let source = r#"
export def my-command [] {
    echo "hello"
}
"#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);
        assert!(
            !violations.is_empty(),
            "Should detect exported function without docs"
        );
        assert_eq!(violations[0].rule_id, "exported_function_docs");
    });
}

#[test]
fn test_exported_function_with_params_without_docs() {
    let source = r"
export def process-data [input: string, output: string] {
    echo $input | save $output
}
";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = (rule().check)(&context);
        assert!(
            !violations.is_empty(),
            "Should detect exported function with params without docs"
        );
        assert_eq!(violations[0].rule_id, "exported_function_docs");
    });
}
