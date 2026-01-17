use super::RULE;

#[test]
fn test_missing_help_field() {
    let bad_code = r#"
def validate [input: string] {
    if ($input | is-empty) {
        error make { msg: "Input cannot be empty" }
    }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_missing_help_with_labels_present() {
    let bad_code = r#"
def process [data] {
    error make {
        msg: "Invalid data format"
        labels: { text: "here", span: (metadata $data).span }
    }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_multiple_errors_missing_help() {
    let bad_code = r#"
def validate-all [a: int, b: int] {
    if $a < 0 {
        error make { msg: "a must be positive" }
    }
    if $b < 0 {
        error make { msg: "b must be positive" }
    }
}
"#;
    RULE.assert_count(bad_code, 2);
}
