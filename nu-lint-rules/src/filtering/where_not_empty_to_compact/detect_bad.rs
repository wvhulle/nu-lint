use super::RULE;

#[test]
fn detect_empty_string_on_right() {
    RULE.assert_detects(r#"["a" ""] | where $it != """#);
}

#[test]
fn detect_single_quoted_empty_string() {
    RULE.assert_detects(r#"["a" ""] | where $it != ''"#);
}
