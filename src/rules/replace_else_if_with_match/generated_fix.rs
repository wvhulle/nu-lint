use super::RULE;
use crate::log::instrument;

#[test]
fn test_simple_string_chain_fix() {
    instrument();
    let bad_code = r#"
if $status == "ok" {
    "success"
} else if $status == "error" {
    "failed"
} else if $status == "pending" {
    "waiting"
} else {
    "unknown"
}
"#;
    RULE.assert_replacement_contains(bad_code, "match $status");
    RULE.assert_replacement_contains(bad_code, r#""ok" =>"#);
    RULE.assert_replacement_contains(bad_code, r#""error" =>"#);
    RULE.assert_replacement_contains(bad_code, r#""pending" =>"#);
    RULE.assert_replacement_contains(bad_code, "_ =>");
    RULE.assert_replacement_contains(bad_code, r#""success""#);
    RULE.assert_replacement_contains(bad_code, r#""failed""#);
}

#[test]
fn test_numeric_chain_fix() {
    let bad_code = r#"
if $code == 200 {
    "ok"
} else if $code == 404 {
    "not found"
} else if $code == 500 {
    "error"
} else {
    "unknown"
}
"#;
    RULE.assert_replacement_contains(bad_code, "match $code");
    RULE.assert_replacement_contains(bad_code, "200 =>");
    RULE.assert_replacement_contains(bad_code, "404 =>");
    RULE.assert_replacement_contains(bad_code, "500 =>");
    RULE.assert_replacement_contains(bad_code, "_ =>");
}
