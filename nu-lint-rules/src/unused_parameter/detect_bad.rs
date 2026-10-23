use super::RULE;

#[test]
fn test_unused_required_param() {
    let code = r#"
def foo [x: int] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_unused_optional_param() {
    let code = r#"
def foo [x?: int] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_unused_flag() {
    let code = r#"
def foo [--verbose] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_unused_rest_param() {
    let code = r#"
def foo [...rest] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_multiple_unused() {
    let code = r#"
def foo [a, b, c] {
    print "no params used"
}
"#;
    RULE.assert_count(code, 3);
}

#[test]
fn test_some_unused() {
    let code = r#"
def foo [used, unused] {
    print $used
}
"#;
    RULE.assert_count(code, 1);
}

#[test]
fn test_unused_flag_with_value() {
    let code = r#"
def foo [--config: string] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_unused_flag_with_short() {
    let code = r#"
def foo [--verbose (-v)] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_unused_optional_with_default() {
    let code = r#"
def foo [x: int = 42] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_unused_middle_param() {
    let code = r#"
def foo [a, unused, b] {
    print $a
    print $b
}
"#;
    RULE.assert_count(code, 1);
}

#[test]
fn test_unused_in_subcommand() {
    let code = r#"
def "main sub" [unused] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn test_unused_typed_rest() {
    let code = r#"
def foo [...rest: string] {
    print "hello"
}
"#;
    RULE.assert_detects(code);
}
