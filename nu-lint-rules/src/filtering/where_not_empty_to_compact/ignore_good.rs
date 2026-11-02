use super::RULE;

#[test]
fn ignore_compact_empty() {
    RULE.assert_ignores(r#"["a" ""] | compact --empty"#);
}

#[test]
fn ignore_non_empty_string() {
    RULE.assert_ignores(r#"["a" "b"] | where $it != "b""#);
}

#[test]
fn ignore_field_comparison() {
    RULE.assert_ignores(r#"[{a: ""}] | where a != """#);
}

#[test]
fn ignore_null_comparison() {
    RULE.assert_ignores("[1 null] | where $it != null");
}
