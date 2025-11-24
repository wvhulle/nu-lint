use super::rule;

#[test]
fn test_command_with_ignore() {
    let bad_code = "ls | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_complex_pipeline_with_output() {
    let bad_code = "ls | where name =~ 'test' | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_in_function() {
    let bad_code = r"
def fetch_data [] {
    http get https://api.example.com | ignore
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_in_closure() {
    let bad_code = r#"
[1 2 3] | each { |x| echo $"Item ($x)" | ignore }
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn test_external_command_with_ignore() {
    // External commands are handled by external_command_ignore rule
    let good_code = "^ls -la | ignore";
    rule().assert_detects(good_code);
}
