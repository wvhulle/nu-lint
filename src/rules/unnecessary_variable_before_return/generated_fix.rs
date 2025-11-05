use super::rule;

#[test]
fn test_detect_unnecessary_variable_simple() {
    crate::log::instrument();

    let bad_code = r"
def foo [] {
  let result = (some | pipeline)
  $result
}
";

    rule().assert_detects(bad_code);
}
