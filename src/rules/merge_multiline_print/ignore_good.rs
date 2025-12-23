use super::rule;

#[test]
fn test_ignore_single_print() {
    let good_code = r#"print "single line""#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_two_prints() {
    let good_code = r#"
print "line 1"
print "line 2"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_multiline_string() {
    let good_code = r#"
print "line 1
line 2
line 3"
"#;
    rule().assert_ignores(good_code);
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
    rule().assert_ignores(good_code);
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
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_mixed_stderr_stdout() {
    let good_code = r#"
print "stdout 1"
print -e "stderr"
print "stdout 2"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_print_with_pipeline() {
    let good_code = r#"
print "header"
ls | print
print "footer"
"#;
    rule().assert_ignores(good_code);
}
