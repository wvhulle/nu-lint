use super::RULE;

#[test]
fn test_detect_source_current_file() {
    let bad_code = r#"source $env.CURRENT_FILE"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_source_nu_current_file() {
    let bad_code = r#"source $nu.current-file"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_source_in_function() {
    let bad_code = r#"
def reload [] {
    source $env.CURRENT_FILE
}
"#;
    RULE.assert_detects(bad_code);
}
