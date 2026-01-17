use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_detect_equal_null_pattern() {
    init_env_log();
    let bad_code = r#"
def test [x] {
    if $x == null { "default" } else { $x }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_not_equal_null_pattern() {
    let bad_code = r#"
def test [x] {
    if $x != null { $x } else { "default" }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_null_on_left() {
    let bad_code = r#"
def test [value] {
    if null == $value { 0 } else { $value }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_with_numeric_default() {
    let bad_code = r#"
def test [count] {
    if $count == null { 0 } else { $count }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_with_list_default() {
    let bad_code = r#"
def test [items] {
    if $items == null { [] } else { $items }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_with_record_default() {
    let bad_code = r#"
def test [config] {
    if $config == null { {} } else { $config }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_let_binding() {
    let bad_code = r#"
def test [input] {
    let result = if $input == null { "none" } else { $input }
}
"#;
    RULE.assert_detects(bad_code);
}
