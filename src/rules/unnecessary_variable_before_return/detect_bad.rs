#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::unnecessary_variable_before_return::UnnecessaryVariableBeforeReturn;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_unnecessary_variable_with_pipeline() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let bad_code = r"
def get-value [] {
  let result = (some | pipeline)
  $result
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect unnecessary variable before return with pipeline"
        );
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
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect unnecessary variable before return with conversion"
        );
    }
}
