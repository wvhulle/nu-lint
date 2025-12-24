use super::RULE;

#[test]
fn test_detect_error_keyword_in_msg() {
    let bad_code = r#"
def process-file [file: string] {
    if not ($file | path exists) {
        error make { msg: "error" }
    }
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_failed_keyword_in_msg() {
    let bad_code = r#"
def convert-data [input] {
    if ($input | is-empty) {
        error make { msg: "failed" }
    }
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_vague_something_went_wrong_msg() {
    let bad_code = r#"
def validate [data] {
    error make { msg: "something went wrong" }
}
"#;

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}
