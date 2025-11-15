use super::rule;

#[test]
fn test_command_with_no_output_piped_to_ignore() {
    let bad_code = "mkdir /tmp/test | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_in_function() {
    let bad_code = r"
def setup [] {
    mkdir /tmp/data | ignore
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_in_closure() {
    let bad_code = r#"
[1 2 3] | each { |x| mkdir $"/tmp/dir($x)" | ignore }
"#;
    rule().assert_detects(bad_code);
}
