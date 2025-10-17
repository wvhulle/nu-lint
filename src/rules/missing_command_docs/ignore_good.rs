#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::Rule, rules::missing_command_docs::MissingCommandDocs,
    };

    #[test]
    fn test_command_with_docs_not_flagged() {
        let rule = MissingCommandDocs::new();

        let good_code = r"
# This is a documented command
def my-command [] {
    echo 'hello'
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(violations.is_empty(), "Should not flag documented commands");
        });
    }

    #[test]
    fn test_command_with_multiline_docs_not_flagged() {
        let rule = MissingCommandDocs::new();

        let good_code = r"
# Process some data
# Takes input and processes it
def process-data [input: string] {
    print $input
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag commands with multiline docs"
            );
        });
    }

    #[test]
    fn test_command_with_doc_comment_not_flagged() {
        let rule = MissingCommandDocs::new();

        let good_code = r#"
# A simple greeting command
def greet [name: string] {
    print $"Hello ($name)"
}
"#;

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag commands with doc comments"
            );
        });
    }
}
