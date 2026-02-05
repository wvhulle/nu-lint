use super::RULE;

#[test]
fn test_single_assignment() {
    RULE.assert_ignores(
        r#"
mut config = {}
$config.debug = true
"#,
    );
}

#[test]
fn test_env_assignments() {
    // $env should be handled by use_load_env, not this rule
    RULE.assert_ignores(
        r#"
$env.VAR1 = "a"
$env.VAR2 = "b"
"#,
    );
}

#[test]
fn test_nested_paths() {
    // Nested paths are no longer linted
    RULE.assert_ignores(
        r#"
mut config = {}
$config.db.host = "localhost"
$config.db.port = 5432
"#,
    );
}

#[test]
fn test_different_roots() {
    RULE.assert_ignores(
        r#"
mut config = {}
mut data = {}
$config.a = 1
$data.b = 2
"#,
    );
}

#[test]
fn test_no_conflict_with_use_load_env() {
    // Flat $env should not trigger use_record_spread
    RULE.assert_count(
        r#"
$env.VAR1 = "value1"
$env.VAR2 = "value2"
"#,
        0,
    );
}

#[test]
fn test_nested_non_env_not_detected() {
    // Nested non-$env should not trigger use_record_spread
    RULE.assert_count(
        r#"
mut cfg = {}
$cfg.server.host = "localhost"
$cfg.server.port = 8080
"#,
        0,
    );
}
