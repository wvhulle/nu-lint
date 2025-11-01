use super::rule;
use crate::LintContext;

#[test]
fn test_detect_unnecessary_variable_simple() {
    crate::log::instrument();

    let bad_code = r"
def foo [] {
  let result = (some | pipeline)
  $result
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        let violations = (rule().check)(&context);
        assert!(!violations.is_empty(), "Should detect unnecessary variable");
        assert!(
            violations[0].message.contains("result"),
            "Message should mention the variable name"
        );
    });
}
