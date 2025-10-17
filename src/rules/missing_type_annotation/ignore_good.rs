#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::Rule, rules::missing_type_annotation::MissingTypeAnnotation,
    };

    #[test]
    fn test_ignore_fully_annotated_params() {
        let rule = MissingTypeAnnotation::new();

        let good_code = r#"
def greet [name: string] {
    print $"Hello ($name)"
}
"#;

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag parameters with type annotations"
            );
        });
    }

    #[test]
    fn test_ignore_multiple_annotated_params() {
        let rule = MissingTypeAnnotation::new();

        let good_code = r"
def add [x: int, y: int] {
    $x + $y
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag parameters with type annotations"
            );
        });
    }

    #[test]
    fn test_ignore_flags() {
        let rule = MissingTypeAnnotation::new();

        let good_code = r"
def process [
    input: string
    --verbose
    --output: string
] {
    print $input
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag flags or annotated parameters"
            );
        });
    }

    #[test]
    fn test_ignore_spread_params() {
        let rule = MissingTypeAnnotation::new();

        let good_code = r"
def variadic [...args: list] {
    print $args
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag spread parameters with annotations"
            );
        });
    }

    #[test]
    fn test_ignore_no_params() {
        let rule = MissingTypeAnnotation::new();

        let good_code = r"
def hello [] {
    print 'Hello world'
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag functions with no parameters"
            );
        });
    }

    #[test]
    fn test_ignore_complex_types() {
        let rule = MissingTypeAnnotation::new();

        let good_code = r"
def process [
    data: list<string>
    options: record
] {
    print $data
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag parameters with complex type annotations"
            );
        });
    }
}
