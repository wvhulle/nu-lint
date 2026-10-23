use super::RULE;

#[test]
fn test_used_param() {
    let code = r#"
def foo [x: int] {
    print $x
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_underscore_prefix() {
    let code = r#"
def foo [_unused: int] {
    print "hello"
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_all_params_used() {
    let code = r#"
def add [a: int, b: int] {
    $a + $b
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_used_flag() {
    let code = r#"
def foo [--verbose] {
    if $verbose { print "verbose mode" }
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_used_rest_param() {
    let code = r#"
def foo [...args] {
    $args | each { |x| print $x }
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_used_in_cell_path() {
    let code = r#"
def foo [record] {
    $record.field
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_used_in_nested_block() {
    let code = r#"
def foo [x] {
    if true {
        print $x
    }
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_used_in_closure() {
    let code = r#"
def foo [x] {
    [1 2 3] | each { |_| $x }
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_used_in_string_interpolation() {
    let code = r#"
def greet [name] {
    $"Hello, ($name)!"
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_used_in_match() {
    let code = r#"
def foo [x] {
    match $x {
        1 => "one"
        _ => "other"
    }
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_flag_with_value_used() {
    let code = r#"
def foo [--config: string] {
    open $config
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_optional_param_used() {
    let code = r#"
def foo [x?: int] {
    $x | default 0
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_used_in_pipeline() {
    let code = r#"
def foo [data] {
    $data | where size > 0 | first
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_passed_to_other_function() {
    let code = r#"
def helper [x] { $x }
def foo [x] {
    helper $x
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_rest_param_spread() {
    let code = r#"
def foo [...args] {
    other ...$args
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_param_used_in_record_literal() {
    let code = r#"
def foo [name, value] {
    { name: $name, value: $value }
}
"#;
    RULE.assert_ignores(code);
}

#[test]
fn test_builtin_help_flag_ignored() {
    // --help is auto-added and should not be flagged
    let code = r#"
def foo [] {
    print "hello"
}
"#;
    RULE.assert_ignores(code);
}
