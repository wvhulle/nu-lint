use super::rule;

#[test]
fn test_detect_unnecessary_variable_with_pipeline() {
    let bad_code = r"
def get-value [] {
  let result = (some | pipeline)
  $result
}
";

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
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
    rule().assert_violation_count_exact(bad_code, 1);
}
