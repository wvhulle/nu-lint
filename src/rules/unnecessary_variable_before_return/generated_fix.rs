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
    rule().assert_help_contains(bad_code, "Return the expression directly");
    rule().assert_help_contains(bad_code, "instead of assigning to a variable first");
    rule().assert_replacement_contains(bad_code, "some | pipeline");
}

#[test]
fn test_fix_simple_variable() {
    let bad_code = r"
def foo [] {
  let x = 5
  $x
}
";

    rule().assert_replacement_contains(bad_code, "5");
}

#[test]
fn test_fix_pipeline_expression() {
    let bad_code = r"
def get_data [] {
  let data = (ls | where size > 100kb)
  $data
}
";

    rule().assert_replacement_contains(bad_code, "ls | where size > 100kb");
}

#[test]
fn test_fix_string_expression() {
    let bad_code = r#"
def message [] {
  let msg = "hello"
  $msg
}
"#;

    rule().assert_replacement_contains(bad_code, r#""hello""#);
}

#[test]
fn test_fix_explanation() {
    let bad_code = r"
def foo [] {
  let result = (some | pipeline)
  $result
}
";

    rule().assert_fix_explanation_contains(bad_code, "Return expression directly");
}

#[test]
fn test_fix_record_expression() {
    let bad_code = r"
def config [] {
  let settings = {name: 'app', version: '1.0'}
  $settings
}
";

    rule().assert_replacement_contains(bad_code, "{name: 'app', version: '1.0'}");
}

#[test]
fn test_fix_list_expression() {
    let bad_code = r"
def items [] {
  let list = [1, 2, 3]
  $list
}
";

    rule().assert_replacement_contains(bad_code, "[1, 2, 3]");
}
