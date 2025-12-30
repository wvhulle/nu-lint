use super::RULE;
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

    RULE.assert_detects(bad_code);
    RULE.assert_help_contains(bad_code, "Return the expression directly");
    RULE.assert_help_contains(bad_code, "instead of assigning to a variable first");
    RULE.assert_fixed_contains(bad_code, "some | pipeline");
}

#[test]
fn test_fix_simple_variable() {
    let bad_code = r"
def foo [] {
  let x = 5
  $x
}
";

    RULE.assert_fixed_contains(bad_code, "5");
}

#[test]
fn test_fix_pipeline_expression() {
    let bad_code = r"
def get_data [] {
  let data = (ls | where size > 100kb)
  $data
}
";

    RULE.assert_fixed_contains(bad_code, "ls | where size > 100kb");
}

#[test]
fn test_fix_string_expression() {
    let bad_code = r#"
def message [] {
  let msg = "hello"
  $msg
}
"#;

    RULE.assert_fixed_contains(bad_code, r#""hello""#);
}

#[test]
fn test_fix_explanation() {
    let bad_code = r"
def foo [] {
  let result = (some | pipeline)
  $result
}
";

    RULE.assert_fix_explanation_contains(bad_code, "Return expression directly");
}

#[test]
fn test_fix_record_expression() {
    let bad_code = r"
def config [] {
  let settings = {name: 'app', version: '1.0'}
  $settings
}
";

    RULE.assert_fixed_contains(bad_code, "{name: 'app', version: '1.0'}");
}

#[test]
fn test_fix_list_expression() {
    let bad_code = r"
def items [] {
  let list = [1, 2, 3]
  $list
}
";

    RULE.assert_fixed_contains(bad_code, "[1, 2, 3]");
}
