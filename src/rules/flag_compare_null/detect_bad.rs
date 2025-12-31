use super::RULE;
use crate::log::instrument;

#[test]
fn flag_used_without_null_check() {
    instrument();
    // Typed flags (with : type) can be null and require null checks
    let bad_code = r#"
def my-command [--verbose: string] {
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
    // Typed flags can be null
    let bad_code = r#"
def my-command [--verbose: string --debug: string] {
    if $verbose { print "Verbose" }
    if $debug { print "Debug" }
}
"#;
    RULE.assert_count(bad_code, 2);
}

#[test]
fn flag_with_short_form_used_without_null_check() {
    instrument();
    // Typed flags with short form can be null
    let bad_code = r#"
def my-command [--verbose (-v): string] {
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

#[test]
fn flag_with_null_check_but_also_used_without() {
    instrument();
    // Typed flags can be null
    let bad_code = r#"
def my-command [--verbose: string] {
    if $verbose != null {
        print "Verbose mode"
    }
    print $verbose
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_used_in_condition_before_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--count: int] {
    if $count > 0 {
        print "positive"
    }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_used_in_string_interpolation_without_check() {
    instrument();
    let bad_code = r#"
def my-command [--name: string] {
    print $"Hello ($name)"
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_used_in_list_without_check() {
    instrument();
    let bad_code = r#"
def my-command [--item: string] {
    [$item "other"]
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn flag_in_binary_comparison_without_null_check() {
    instrument();
    let bad_code = r#"
def my-command [--value: int] {
    if $value == 42 {
        print "found it"
    }
}
"#;
    RULE.assert_detects(bad_code);
}
