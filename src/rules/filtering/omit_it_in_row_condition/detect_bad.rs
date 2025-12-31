use super::RULE;
use crate::log::init_env_log;

#[test]
fn detect_simple_field_comparison() {
    init_env_log();
    RULE.assert_detects(r#"ls | where $it.type == "dir""#);
}

#[test]
fn detect_size_comparison() {
    RULE.assert_detects(r"ls | where $it.size > 100kb");
}

#[test]
fn detect_name_comparison() {
    RULE.assert_detects(r#"ls | where $it.name == "foo""#);
}

#[test]
fn detect_field_with_underscore() {
    RULE.assert_detects(r"open data.json | where $it.first_name == 'John'");
}

#[test]
fn detect_multiple_it_field_accesses() {
    let code = r#"ls | where $it.size > 100kb and $it.type == "file""#;
    RULE.assert_count(code, 2);
}

#[test]
fn detect_numeric_comparison() {
    RULE.assert_detects(r"[{x: 1}, {x: 2}] | where $it.x > 1");
}

#[test]
fn detect_string_contains() {
    RULE.assert_detects(r#"ls | where $it.name =~ "test""#);
}

#[test]
fn detect_boolean_field() {
    RULE.assert_detects(r"open data.json | where $it.active == true");
}

#[test]
fn detect_date_comparison() {
    RULE.assert_detects(r"ls | where $it.modified >= (date now) - 2wk");
}

#[test]
fn detect_in_function() {
    let code = r"
def filter_large [] {
    ls | where $it.size > 1kb
}
";
    RULE.assert_detects(code);
}
