#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::Rule, rules::missing_command_docs::MissingCommandDocs,
    };

    #[test]
    fn test_command_without_docs_detected() {
        let rule = MissingCommandDocs::new();

        let bad_code = r"
def my-command [] {
    echo 'hello'
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect missing command docs");
            assert!(violations[0].message.contains("my-command"));
        });
    }

    #[test]
    fn test_command_with_params_without_docs_detected() {
        let rule = MissingCommandDocs::new();

        let bad_code = r"
def process-data [input: string, --verbose] {
    print $input
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect missing command docs");
            assert!(violations[0].message.contains("process-data"));
        });
    }

    #[test]
    fn test_multiple_commands_without_docs_detected() {
        let rule = MissingCommandDocs::new();

        let bad_code = r"
def first-command [] {
    echo 'first'
}

def second-command [] {
    echo 'second'
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                2,
                "Should detect both undocumented commands"
            );
        });
    }
}
