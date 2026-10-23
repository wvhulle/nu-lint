use super::RULE;

#[test]
fn test_labels_with_span() {
    let good_code = r#"
def validate [input: string] {
    if ($input | is-empty) {
        error make {
            msg: "Input cannot be empty"
            labels: { text: "empty input", span: (metadata $input).span }
        }
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_no_labels_field() {
    let good_code = r#"
def validate [input: string] {
    error make { msg: "Input cannot be empty" }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_labels_with_all_fields() {
    let good_code = r#"
def check-file [path: string] {
    if not ($path | path exists) {
        error make {
            msg: $"File not found: ($path)"
            labels: [{ text: "file does not exist", span: (metadata $path).span }]
            help: "Check the file path and try again"
        }
    }
}
"#;
    RULE.assert_ignores(good_code);
}
