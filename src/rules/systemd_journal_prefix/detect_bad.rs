use super::rule;

#[test]
fn test_detect_print_without_prefix() {
    let bad_code = r#"print "Hello, World!""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_without_prefix() {
    let bad_code = r#"echo "Starting process""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_single_quotes() {
    let bad_code = r"print 'Error occurred'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_single_quotes() {
    let bad_code = r"echo 'Process completed'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_prints_without_prefix() {
    let bad_code = r#"
print "Starting task"
print "Task completed"
"#;
    rule().assert_violation_count(bad_code, 2);
}

#[test]
fn test_detect_print_in_function() {
    let bad_code = r#"
def deploy [] {
    print "Deploying application"
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_echo_in_function() {
    let bad_code = r#"
def backup [] {
    echo "Backing up files"
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_print_after_semicolon() {
    let bad_code = r#"let x = 5; print "Value set""#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_mixed_print_echo() {
    let bad_code = r#"
print "Step 1"
echo "Step 2"
"#;
    rule().assert_violation_count(bad_code, 2);
}

#[test]
fn test_detect_print_with_variable() {
    let bad_code = r#"print $"Processing {$file}""#;
    rule().assert_detects(bad_code);
}
