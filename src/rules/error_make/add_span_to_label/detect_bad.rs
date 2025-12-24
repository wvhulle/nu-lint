use super::RULE;

#[test]
fn test_labels_missing_span() {
    let bad_code = r#"
def process [data: record] {
    error make {
        msg: "Invalid data format"
        labels: { text: "problematic field" }
    }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_labels_missing_span_with_help() {
    let bad_code = r#"
def validate [input] {
    error make {
        msg: "Validation failed"
        labels: { text: "invalid value" }
        help: "Check the input format"
    }
}
"#;
    RULE.assert_detects(bad_code);
}
