use super::RULE;

#[test]
fn test_single_nested_assignment() {
    RULE.assert_ignores(
        r#"
mut config = {}
$config.db.host = "localhost"
"#,
    );
}

#[test]
fn test_flat_assignments() {
    // Flat paths should be handled by merge_flat_upsert
    RULE.assert_ignores(
        r#"
mut config = {}
$config.a = 1
$config.b = 2
"#,
    );
}

#[test]
fn test_flat_env() {
    // Flat $env should be handled by use_load_env
    RULE.assert_ignores(
        r#"
$env.VAR1 = "a"
$env.VAR2 = "b"
"#,
    );
}

#[test]
fn test_nested_env() {
    // Nested $env can't be merged - can't reassign $env directly
    RULE.assert_ignores(
        r#"
$env.config.show_banner = false
$env.config.buffer_editor = "vim"
"#,
    );
}

#[test]
fn test_no_conflict_with_use_load_env() {
    // Flat $env should not trigger merge_nested_upsert
    RULE.assert_count(
        r#"
$env.PATH = "/usr/bin"
$env.HOME = "/home/user"
"#,
        0,
    );
}

#[test]
fn test_no_conflict_with_merge_flat_upsert() {
    // Flat non-$env should not trigger merge_nested_upsert
    RULE.assert_count(
        r#"
mut data = {}
$data.x = 1
$data.y = 2
"#,
        0,
    );
}

#[test]
fn test_nested_env_not_detected() {
    // Nested $env should not be detected (can't reassign $env)
    RULE.assert_count(
        r#"
$env.config.theme = "dark"
$env.config.font = "monospace"
"#,
        0,
    );
}
