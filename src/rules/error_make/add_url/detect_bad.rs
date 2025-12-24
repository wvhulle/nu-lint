use super::RULE;

#[test]
fn test_missing_url_field() {
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
fn test_missing_url_with_other_fields() {
    let bad_code = r#"
def check [value: int] {
    if $value < 0 {
        error make {
            msg: "Value must be non-negative"
            labels: { text: "negative value", span: (metadata $value).span }
            help: "Provide a positive integer"
        }
    }
}
"#;
    RULE.assert_detects(bad_code);
}
