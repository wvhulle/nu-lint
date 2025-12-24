use super::RULE;
use crate::log::instrument;

#[test]
fn ignore_nushell_find() {
    instrument();
    RULE.assert_ignores(r#"find \"pattern\""#);
}

#[test]
fn ignore_nushell_where_pipeline() {
    instrument();
    RULE.assert_ignores(r#"lines | where $it =~ \"pattern\""#);
}

#[test]
fn ignore_nushell_str_contains() {
    instrument();
    RULE.assert_ignores(r#"lines | where $it | str contains \"text\""#);
}

#[test]
fn ignore_http_get() {
    instrument();
    RULE.assert_ignores(r"http get https://example.com");
}
