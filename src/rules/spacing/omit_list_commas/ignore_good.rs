use super::RULE;
use crate::log::init_log;

#[test]
fn comma_in_comment() {
    init_log();
    let code = r#"let responses = [
      "item1"
      # Batch dedup (discard 2,3)   <-- flagged as list comma
      "item2"
  ]"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_list_without_commas() {
    let code = "let items = [1 2 3]";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_empty_list() {
    let code = "let empty = []";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_single_item_list() {
    let code = "let single = [42]";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_multiline_list_without_commas() {
    let code = r#"let items = [
    "first"
    "second"
    "third"
]"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_comma_inside_comment() {
    let code = r#"let responses = [
    "item1"
    # Batch dedup (discard 2,3)
    "item2"
]"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_comma_in_inline_comment() {
    let code = r#"let items = [
    "first" # inline comment with comma, here
    "second"
]"#;
    RULE.assert_ignores(code);
}
