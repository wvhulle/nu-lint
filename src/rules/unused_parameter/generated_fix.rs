use super::RULE;

// Tests for exported functions (should prefix with underscore)

#[test]
fn test_fix_exported_prefixes_with_underscore() {
    let code = r#"export def foo [unused: int] {
    print "hello"
}"#;
    RULE.assert_fixed_contains(code, "_unused");
}

#[test]
fn test_fix_main_prefixes_with_underscore() {
    let code = r#"def main [unused: int] {
    print "hello"
}"#;
    RULE.assert_fixed_contains(code, "_unused");
}

#[test]
fn test_fix_exported_flag() {
    let code = r#"export def foo [--verbose] {
    print "hello"
}"#;
    RULE.assert_fixed_contains(code, "_verbose");
}

// Tests for non-exported functions (should remove parameter and update call
// sites)

#[test]
fn test_fix_removes_unused_required() {
    let code = r#"def foo [x] {
    print "no x"
}
def main [] {
    foo "arg"
}"#;
    // Note: assert_fixed_contains is more lenient with whitespace
    RULE.assert_fixed_contains(code, "def foo []");
    // Verify x parameter was removed from signature (not from string "no x")
    RULE.assert_fixed_not_contains(code, "[x]");
}

#[test]
fn test_fix_removes_unused_preserves_used() {
    let code = r#"def foo [used, unused] {
    print $used
}
def main [] {
    foo "a" "b"
}"#;
    let expected = r#"def foo [used] {
    print $used
}
def main [] {
    foo "a"
}"#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn test_fix_removes_first_param() {
    let code = r#"def foo [unused, used] {
    print $used
}
def main [] {
    foo "a" "b"
}"#;
    let expected = r#"def foo [used] {
    print $used
}
def main [] {
    foo "b"
}"#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn test_fix_removes_flag() {
    let code = r#"def helper [--verbose] {
    print "hi"
}
def main [] {
    helper --verbose
}"#;
    RULE.assert_fixed_not_contains(code, "--verbose");
}

#[test]
fn test_fix_removes_rest_and_trailing_args() {
    let code = r#"def helper [...rest] {
    print "hi"
}
def main [] {
    helper 1 2 3
}"#;
    // Note: assert_fixed_contains is more lenient with whitespace
    RULE.assert_fixed_contains(code, "def helper []");
    RULE.assert_fixed_not_contains(code, "...rest");
}

// Edge cases for fix behavior

#[test]
fn test_fix_multiple_call_sites() {
    let code = r#"def helper [unused] {
    print "hi"
}
def main [] {
    helper "a"
    helper "b"
    helper "c"
}"#;
    RULE.assert_fixed_contains(code, "def helper []");
    // All call sites should have argument removed
    RULE.assert_fixed_not_contains(code, r#"helper "a""#);
}

#[test]
fn test_fix_typed_param() {
    let code = r#"def helper [unused: int] {
    print "hi"
}
def main [] {
    helper 42
}"#;
    RULE.assert_fixed_contains(code, "def helper []");
}

#[test]
fn test_fix_middle_param_of_three() {
    let code = r#"def helper [a, unused, b] {
    print $a
    print $b
}
def main [] {
    helper 1 2 3
}"#;
    RULE.assert_fixed_contains(code, "def helper [a, b]");
}

#[test]
fn test_fix_flag_with_value() {
    let code = r#"def helper [--config: string] {
    print "hi"
}
def main [] {
    helper --config "path"
}"#;
    RULE.assert_fixed_not_contains(code, "--config");
}

#[test]
fn test_fix_preserves_other_flags() {
    let code = r#"def helper [--verbose, --unused] {
    if $verbose { print "v" }
}
def main [] {
    helper --verbose --unused
}"#;
    RULE.assert_fixed_contains(code, "--verbose");
    RULE.assert_fixed_not_contains(code, "--unused");
}

#[test]
fn test_fix_subcommand_exported() {
    // Subcommands under main are treated as entry points
    let code = r#"def "main sub" [unused] {
    print "hi"
}"#;
    RULE.assert_fixed_contains(code, "_unused");
}

#[test]
fn test_fix_rest_with_type() {
    let code = r#"def helper [...rest: string] {
    print "hi"
}
def main [] {
    helper "a" "b"
}"#;
    RULE.assert_fixed_contains(code, "def helper []");
}

#[test]
fn test_fix_optional_param() {
    let code = r#"def helper [x?: int] {
    print "hi"
}
def main [] {
    helper 42
}"#;
    RULE.assert_fixed_contains(code, "def helper []");
}

#[test]
fn test_fix_no_call_sites() {
    // Helper with no calls - just remove param from signature
    let code = r#"def helper [unused] {
    print "hi"
}
def main [] {
    print "no calls"
}"#;
    RULE.assert_fixed_contains(code, "def helper []");
}
