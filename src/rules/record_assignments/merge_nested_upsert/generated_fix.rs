use super::RULE;

#[test]
fn test_fix_nested_same_parent() {
    RULE.assert_fixed_is(
        r#"mut config = {}
$config.db.host = "localhost"
$config.db.port = 5432"#,
        r#"mut config = {}
$config = ($config | upsert db.host "localhost" | upsert db.port 5432)"#,
    );
}

#[test]
fn test_fix_mixed_depths() {
    RULE.assert_fixed_is(
        r#"mut config = {}
$config.debug = true
$config.db.host = "localhost""#,
        r#"mut config = {}
$config = ($config | upsert debug true | upsert db.host "localhost")"#,
    );
}

#[test]
fn test_fix_deeply_nested() {
    RULE.assert_fixed_is(
        r##"mut state = {}
$state.ui.theme.colors.primary = "#007bff"
$state.ui.theme.colors.secondary = "#6c757d""##,
        r##"mut state = {}
$state = ($state | upsert ui.theme.colors.primary "#007bff" | upsert ui.theme.colors.secondary "#6c757d")"##,
    );
}

#[test]
fn test_fix_different_branches() {
    RULE.assert_fixed_is(
        r#"mut app = {}
$app.server.port = 3000
$app.client.timeout = 5000"#,
        r#"mut app = {}
$app = ($app | upsert server.port 3000 | upsert client.timeout 5000)"#,
    );
}

#[test]
fn test_fix_three_nested() {
    RULE.assert_fixed_is(
        r#"mut cfg = {}
$cfg.a.x = 1
$cfg.b.y = 2
$cfg.c.z = 3"#,
        r#"mut cfg = {}
$cfg = ($cfg | upsert a.x 1 | upsert b.y 2 | upsert c.z 3)"#,
    );
}
