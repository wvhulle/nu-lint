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

#[test]
fn test_detect_in_closure() {
    let bad_code = r"
let fn = {||
  let x = (1 + 2)
  $x
}
";

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_with_complex_expression() {
    let bad_code = r"
def process [] {
  let data = (http get 'https://api.example.com' | from json)
  $data
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_simple_value() {
    let bad_code = r"
def get-number [] {
  let num = (42)
  $num
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_in_nested_function() {
    let bad_code = r"
def outer [] {
  def inner [] {
    let value = (10 * 20)
    $value
  }
  inner
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_violations_in_function() {
    let bad_code = r"
def process [] {
  let x = (1 + 2)
  $x
  
  let y = (3 + 4)
  $y
}
";

    rule().assert_violation_count_exact(bad_code, 2);
}
