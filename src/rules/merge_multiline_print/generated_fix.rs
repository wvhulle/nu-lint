use super::rule;

#[test]
fn test_fix_merges_three_prints() {
    let code = r#"print "a"
print "b"
print "c""#;
    rule().assert_detects(code);
    // The fix merges content with actual newlines for a multiline string
    rule().assert_replacement_contains(code, "\"a\nb\nc\"");
}

#[test]
fn test_fix_preserves_stderr_flag() {
    let code = r#"print -e "error 1"
print -e "error 2"
print -e "error 3""#;
    rule().assert_detects(code);
    rule().assert_replacement_contains(code, "-e");
    rule().assert_replacement_contains(code, "\"error 1\nerror 2\nerror 3\"");
}

#[test]
fn test_fix_simple_strings() {
    let code = r#"print "line one"
print "line two"
print "line three""#;
    rule().assert_detects(code);
    rule().assert_replacement_contains(code, "\"line one\nline two\nline three\"");
}

#[test]
fn test_fix_single_quoted_strings() {
    let code = r#"print 'line one'
print 'line two'
print 'line three'"#;
    rule().assert_detects(code);
    rule().assert_replacement_contains(code, "\"line one\nline two\nline three\"");
}

#[test]
fn test_fix_mixed_quote_styles() {
    let code = r#"print "double quoted"
print 'single quoted'
print "another double""#;
    rule().assert_detects(code);
    rule().assert_replacement_contains(code, "double quoted");
    rule().assert_replacement_contains(code, "single quoted");
}

#[test]
fn test_fix_string_interpolation() {
    let code = r#"let name = "world"
print $"Hello ($name)"
print $"Welcome ($name)"
print $"Goodbye ($name)""#;
    rule().assert_detects(code);
    // Should generate: print $"Hello ($name)\nWelcome ($name)\nGoodbye ($name)"
    rule().assert_replacement_contains(code, "$\"");
    rule().assert_replacement_contains(code, "Hello ($name)\nWelcome ($name)\nGoodbye ($name)");
}

#[test]
fn test_fix_string_interpolation_with_stderr() {
    let code = r#"let err = "oops"
print -e $"Error: ($err)"
print -e $"Details: ($err)"
print -e $"Fix: ($err)""#;
    rule().assert_detects(code);
    rule().assert_replacement_contains(code, "-e");
    rule().assert_replacement_contains(code, r#"$""#);
}
