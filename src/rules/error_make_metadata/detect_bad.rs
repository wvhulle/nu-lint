use super::RULE;

#[test]
fn test_error_make_missing_label_and_help() {
    let bad_code = r#"
def validate [input: string] {
    if ($input | is-empty) {
        error make { msg: "Input cannot be empty" }
    }
}
"#;

    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_error_make_missing_help_only() {
    let bad_code = r#"
def process [data] {
    error make {
        msg: "Invalid data format"
        label: { text: "here", span: (metadata $data).span }
    }
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_error_make_missing_label_only() {
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
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_multiple_error_make_calls_missing_metadata() {
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

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 2);
}

#[test]
fn test_error_make_in_closure() {
    let bad_code = r#"
def process-items [items: list] {
    $items | each { |item|
        if ($item | is-empty) {
            error make { msg: "Item is empty" }
        }
    }
}
"#;

    RULE.assert_detects(bad_code);
}
