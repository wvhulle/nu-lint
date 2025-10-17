#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::AstRule, rules::missing_type_annotation::MissingTypeAnnotation,
    };

    #[test]
    fn test_detect_missing_type_annotation_single_param() {
        let rule = MissingTypeAnnotation::new();

        let bad_code = r#"
def greet [name] {
    print $"Hello ($name)"
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect missing type annotation on 'name' parameter"
            );
            assert!(violations[0].message.contains("name"));
        });
    }

    #[test]
    fn test_detect_missing_type_annotation_multiple_params() {
        let rule = MissingTypeAnnotation::new();

        let bad_code = r"
def add [x, y] {
    $x + $y
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                2,
                "Should detect missing type annotations on both 'x' and 'y' parameters"
            );
        });
    }

    #[test]
    fn test_detect_mixed_annotations() {
        let rule = MissingTypeAnnotation::new();

        let bad_code = r"
def process [data, format: string] {
    print $data
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                1,
                "Should detect missing type annotation only on 'data' parameter"
            );
            assert!(violations[0].message.contains("data"));
        });
    }

    #[test]
    fn test_detect_nested_function() {
        let rule = MissingTypeAnnotation::new();

        let bad_code = r"
def outer [] {
    def inner [param] {
        print $param
    }
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect missing type annotation in nested function"
            );
        });
    }
}
