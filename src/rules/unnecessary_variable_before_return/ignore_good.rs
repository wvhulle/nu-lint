#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext,
        rule::Rule,
        rules::unnecessary_variable_before_return::UnnecessaryVariableBeforeReturn,
    };

    #[test]
    fn test_variable_used_multiple_times_not_flagged() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let good_code = r"
def foo [] {
  let result = (some | pipeline)
  print $result
  $result
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                0,
                "Should not flag variable used multiple times"
            );
        });
    }

    #[test]
    fn test_direct_return_not_flagged() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let good_code = r"
def foo [] {
  some | pipeline
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                0,
                "Should not flag direct return"
            );
        });
    }

    #[test]
    fn test_variable_with_additional_logic_not_flagged() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let good_code = r"
def process [] {
  let data = (load | some | data)
  if ($data | is-empty) {
    error make { msg: 'No data' }
  }
  $data
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                0,
                "Should not flag variable when there's additional logic between assignment and return"
            );
        });
    }

    #[test]
    fn test_variable_assigned_without_parens_not_flagged() {
        let rule = UnnecessaryVariableBeforeReturn::new();

        let good_code = r"
def process [] {
  let result = $input | transform
  $result
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert_eq!(
                violations.len(),
                0,
                "Should not flag when assignment is not wrapped in parentheses"
            );
        });
    }
}
