use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_unnecessary_variable_simple() {
    instrument();

    let bad_code = r"
def foo [] {
  let result = (some | pipeline)
  $result
}
";

    rule().assert_detects(bad_code);
}
