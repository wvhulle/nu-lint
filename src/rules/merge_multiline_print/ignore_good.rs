use super::RULE;

#[test]
fn test_ignore_single_print() {
    let good_code = r#"print "single line""#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_two_prints() {
    let good_code = r#"
print "line 1"
print "line 2"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_multiline_string() {
    let good_code = r#"
print "line 1
line 2
line 3"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_plain_and_interpolation() {
    // Mixed plain strings and interpolation should NOT be merged
    // because they have different string types
    let good_code = r#"
let name = "world"
print $"Hello ($name)"
print "Static line"
print $"Bye ($name)"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_prints_separated_by_other_statements() {
    let good_code = r#"
print "Starting..."
let x = 5
print "Processing..."
let y = 10
print "Done."
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_stderr_stdout() {
    let good_code = r#"
print "stdout 1"
print -e "stderr"
print "stdout 2"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_pipeline() {
    let good_code = r#"
print "header"
ls | print
print "footer"
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_backtick_strings() {
    // Backtick strings have different semantics (used for paths/commands)
    // and should not be merged even if consecutive
    let good_code = r#"
print `line one`
print `line two`
print `line three`
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_double_and_single_quote_interpolations() {
    // $"..." and $'...' interpolations should not be merged together
    // because they use different quote styles
    let good_code = r#"
let x = 5
print $"Double: ($x)"
print $'Single: ($x)'
print $"Double again: ($x)"
"#;
    RULE.assert_ignores(good_code);
}
