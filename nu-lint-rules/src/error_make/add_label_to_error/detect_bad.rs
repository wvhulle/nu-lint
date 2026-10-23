use super::RULE;

#[test]
fn test_missing_labels_field() {
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
fn test_missing_labels_with_help_present() {
    let bad_code = r#"
def check [value: int] {
    if $value < 0 {
        error make {
            msg: "Value must be non-negative"
            help: "Provide a positive integer"
        }
    }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_multiple_errors_missing_labels() {
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
