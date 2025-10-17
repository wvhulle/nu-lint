#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::RegexRule,
        rules::unnecessary_variable_before_return::UnnecessaryVariableBeforeReturn,
    };

    #[test]
    fn test_detect_unnecessary_variable_with_pipeline() {
        let rule = UnnecessaryVariableBeforeReturn::new();
        let bad_code = r"
def get-value [] {
  let result = (some | pipeline)
  $result
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect unnecessary variable before return with pipeline"
            );
        });
    }

    #[test]
    fn test_detect_unnecessary_variable_with_conversion() {
        let rule = UnnecessaryVariableBeforeReturn::new();
        let bad_code = r"
def calculate [] {
  let answer = (42 | into string)
  $answer
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect unnecessary variable before return with conversion"
            );
        });
    }

    #[test]
    fn test_detect_unnecessary_variable_simple() {
        let rule = UnnecessaryVariableBeforeReturn::new();
        let bad_code = r"
def foo [] {
  let result = (some | pipeline)
  $result
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect unnecessary variable");
            assert!(
                violations[0].message.contains("result"),
                "Message should mention the variable name"
            );
        });
    }
}
