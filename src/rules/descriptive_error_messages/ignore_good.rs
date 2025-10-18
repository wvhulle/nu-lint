use super::rule;

#[test]
fn test_descriptive_error_message_not_flagged() {
    let source = r#"
def process [input: int] {
    if $input < 0 {
        error make { msg: "Input must be non-negative, got: " + ($input | into string) }
    }
}
"#;
    rule().assert_ignores(source);
}

#[test]
fn test_specific_error_message_not_flagged() {
    let source = r#"
def validate_file [path: string] {
    if not ($path | path exists) {
        error make { msg: $"File not found: ($path)" }
    }
}
"#;
    rule().assert_ignores(source);
}
