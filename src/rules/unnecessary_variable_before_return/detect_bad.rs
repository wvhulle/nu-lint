use super::rule;
use crate::LintContext;

#[test]
fn test_detect_unnecessary_variable_with_pipeline() {
    let bad_code = r"
def get-value [] {
  let result = (some | pipeline)
  $result
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_unnecessary_variable_with_conversion() {
    let bad_code = r"
def calculate [] {
  let answer = (42 | into string)
  $answer
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_unnecessary_variable_simple() {
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
