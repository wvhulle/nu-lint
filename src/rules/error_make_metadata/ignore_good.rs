use super::RULE;

#[test]
fn test_error_make_with_all_metadata() {
    let good_code = r#"
def validate [input: string] {
    if ($input | is-empty) {
        error make {
            msg: "Input cannot be empty"
            label: { text: "empty input provided", span: (metadata $input).span }
            help: "Provide a non-empty string"
        }
    }
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_error_make_with_label_and_help() {
    let good_code = r#"
def process [data: record] {
    error make {
        msg: "Invalid data format"
        label: { text: "problematic field" }
        help: "Ensure all required fields are present"
    }
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_error_make_with_span_in_label() {
    let good_code = r#"
def check-file [path: string] {
    if not ($path | path exists) {
        error make {
            msg: $"File not found: ($path)"
            label: {
                text: "file does not exist"
                span: (metadata $path).span
            }
            help: "Check the file path and try again"
        }
    }
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_multiple_well_formed_error_make_calls() {
    let good_code = r#"
def validate-range [min: int, max: int] {
    if $min >= $max {
        error make {
            msg: "Minimum must be less than maximum"
            label: { text: "invalid range" }
            help: "Ensure min < max"
        }
    }
    if $min < 0 {
        error make {
            msg: "Minimum cannot be negative"
            label: { text: "negative minimum" }
            help: "Use a non-negative minimum value"
        }
    }
}
"#;

    RULE.assert_ignores(good_code);
}
