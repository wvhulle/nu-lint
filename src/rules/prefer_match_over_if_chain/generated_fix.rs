use super::rule;
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
    rule().assert_fix_contains(bad_code, "match $status");
    rule().assert_fix_contains(bad_code, r#""ok" =>"#);
    rule().assert_fix_contains(bad_code, r#""error" =>"#);
    rule().assert_fix_contains(bad_code, r#""pending" =>"#);
    rule().assert_fix_contains(bad_code, "_ =>");
    rule().assert_fix_contains(bad_code, r#""success""#);
    rule().assert_fix_contains(bad_code, r#""failed""#);
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
    rule().assert_fix_contains(bad_code, "match $code");
    rule().assert_fix_contains(bad_code, "200 =>");
    rule().assert_fix_contains(bad_code, "404 =>");
    rule().assert_fix_contains(bad_code, "500 =>");
    rule().assert_fix_contains(bad_code, "_ =>");
}
