use super::RULE;
use crate::log::init_env_log;

#[test]
fn ignore_nushell_find() {
    init_env_log();
    RULE.assert_ignores(r#"find \"pattern\""#);
}

#[test]
fn ignore_nushell_where_pipeline() {
    init_env_log();
    RULE.assert_ignores(r#"lines | where $it =~ \"pattern\""#);
}

#[test]
fn ignore_nushell_str_contains() {
    init_env_log();
    RULE.assert_ignores(r#"lines | where $it | str contains \"text\""#);
}

#[test]
fn ignore_http_get() {
    init_env_log();
    RULE.assert_ignores(r"http get https://example.com");
}
