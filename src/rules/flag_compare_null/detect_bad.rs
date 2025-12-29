use super::RULE;
use crate::log::instrument;

#[test]
fn flag_used_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--verbose] {
    if $verbose { print "Verbose mode" }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_with_type_used_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--count: int] {
    1..$count
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn multiple_flags_used_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--verbose --debug] {
    if $verbose { print "Verbose" }
    if $debug { print "Debug" }
}
"#;
    RULE.assert_count(bad_code, 2);
}

#[test]
fn flag_with_short_form_used_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--verbose (-v)] {
    if $verbose { print "Verbose mode" }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_used_in_expression_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--count: int] {
    let result = $count + 10
    print $result
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_used_in_pipeline_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--path: string] {
    open $path | lines
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_used_in_record_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--name: string] {
    { user: $name }
}
"#;
    RULE.assert_detects(bad_code);
}
