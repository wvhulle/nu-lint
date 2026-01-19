use super::RULE;

#[test]
fn test_nested_same_parent() {
    RULE.assert_detects(
        r#"
mut config = {}
$config.db.host = "localhost"
$config.db.port = 5432
"#,
    );
}

#[test]
fn test_nested_different_parents() {
    RULE.assert_detects(
        r#"
mut config = {}
$config.db.host = "localhost"
$config.app.name = "myapp"
"#,
    );
}

#[test]
fn test_mixed_depths() {
    RULE.assert_detects(
        r#"
mut config = {}
$config.debug = true
$config.db.host = "localhost"
"#,
    );
}

#[test]
fn test_deeply_nested() {
    RULE.assert_detects(
        r#"
mut data = {}
$data.a.b.c = 1
$data.a.b.d = 2
"#,
    );
}
