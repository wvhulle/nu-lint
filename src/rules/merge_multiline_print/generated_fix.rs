use super::RULE;

#[test]
fn test_fix_merges_three_prints() {
    let code = r#"print "a"
print "b"
print "c""#;
    RULE.assert_detects(code);
    // The fix merges content with actual newlines for a multiline string
    RULE.assert_fixed_contains(code, "\"a\nb\nc\"");
}

#[test]
fn test_fix_preserves_stderr_flag() {
    let code = r#"print -e "error 1"
print -e "error 2"
print -e "error 3""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "-e");
    RULE.assert_fixed_contains(code, "\"error 1\nerror 2\nerror 3\"");
}

#[test]
fn test_fix_simple_strings() {
    let code = r#"print "line one"
print "line two"
print "line three""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "\"line one\nline two\nline three\"");
}

#[test]
fn test_fix_single_quoted_strings() {
    let code = r#"print 'line one'
print 'line two'
print 'line three'"#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "\"line one\nline two\nline three\"");
}

#[test]
fn test_fix_mixed_quote_styles() {
    let code = r#"print "double quoted"
print 'single quoted'
print "another double""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "double quoted");
    RULE.assert_fixed_contains(code, "single quoted");
}

#[test]
fn test_fix_string_interpolation() {
    let code = r#"let name = "world"
print $"Hello ($name)"
print $"Welcome ($name)"
print $"Goodbye ($name)""#;
    RULE.assert_detects(code);
    // Should generate: print $"Hello ($name)\nWelcome ($name)\nGoodbye ($name)"
    RULE.assert_fixed_contains(code, "$\"");
    RULE.assert_fixed_contains(code, "Hello ($name)\nWelcome ($name)\nGoodbye ($name)");
}

#[test]
fn test_fix_string_interpolation_with_stderr() {
    let code = r#"let err = "oops"
print -e $"Error: ($err)"
print -e $"Details: ($err)"
print -e $"Fix: ($err)""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "-e");
    RULE.assert_fixed_contains(code, r#"$""#);
}

#[test]
fn test_fix_strings_containing_quotes() {
    let code = r#"print "She said \"hello\""
print "He replied \"hi\""
print "They shouted \"bye\"""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(
        code,
        "She said \\\"hello\\\"\nHe replied \\\"hi\\\"\nThey shouted \\\"bye\\\"",
    );
}

#[test]
fn test_fix_strings_with_single_quotes_inside() {
    let code = r#"print "It's a test"
print "That's correct"
print "We're done""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "It's a test\nThat's correct\nWe're done");
}

#[test]
fn test_fix_raw_strings() {
    let code = r#"print r#'line one'#
print r#'line two'#
print r#'line three'#"#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "line one\nline two\nline three");
}

#[test]
fn test_fix_raw_strings_with_quotes() {
    let code = r#"print r#'She said "hello"'#
print r#'He replied "hi"'#
print r#'They said "bye"'#"#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(
        code,
        "She said \\\"hello\\\"\nHe replied \\\"hi\\\"\nThey said \\\"bye\\\"",
    );
}

#[test]
fn test_fix_raw_strings_with_single_quotes() {
    let code = r#"print r#'It's working'#
print r#'That's good'#
print r#'We're happy'#"#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "It's working\nThat's good\nWe're happy");
}

#[test]
fn test_fix_string_interpolation_with_multiple_variables() {
    let code = r#"let first = "John"
let last = "Doe"
print $"First: ($first)"
print $"Last: ($last)"
print $"Full: ($first) ($last)""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "$\"");
    RULE.assert_fixed_contains(
        code,
        "First: ($first)\nLast: ($last)\nFull: ($first) ($last)",
    );
}

#[test]
fn test_fix_string_interpolation_with_expressions() {
    let code = r#"let x = 5
print $"Value: ($x)"
print $"Double: ($x * 2)"
print $"Sum: ($x + 10)""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "$\"");
    RULE.assert_fixed_contains(code, "Value: ($x)\nDouble: ($x * 2)\nSum: ($x + 10)");
}

#[test]
fn test_fix_string_interpolation_with_field_access() {
    let code = r#"let record = {name: "test", value: 42}
print $"Name: ($record.name)"
print $"Value: ($record.value)"
print $"Both: ($record.name) = ($record.value)""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "$\"");
    RULE.assert_fixed_contains(
        code,
        "Name: ($record.name)\nValue: ($record.value)\nBoth: ($record.name) = ($record.value)",
    );
}

#[test]
fn test_fix_string_interpolation_mixed_with_text() {
    let code = r#"let count = 3
print $"Processing ($count) items..."
print $"Progress: ($count)/10"
print $"Done with ($count) files""#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "$\"");
    RULE.assert_fixed_contains(
        code,
        "Processing ($count) items...\nProgress: ($count)/10\nDone with ($count) files",
    );
}

#[test]
fn test_fix_string_interpolation_with_single_quotes() {
    let code = r#"let msg = "hello"
print $'Message: ($msg)'
print $'Status: active'
print $'Reply: ($msg) world'"#;
    RULE.assert_detects(code);
    RULE.assert_fixed_contains(code, "$'");
    RULE.assert_fixed_contains(code, "Message: ($msg)\nStatus: active\nReply: ($msg) world");
}
