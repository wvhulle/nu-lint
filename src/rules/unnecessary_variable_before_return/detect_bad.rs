#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::Rule,
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
}
