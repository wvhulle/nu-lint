use super::RULE;

#[test]
fn test_two_flat_assignments() {
    RULE.assert_detects(
        r#"
mut config = {}
$config.debug = true
$config.verbose = false
"#,
    );
}

#[test]
fn test_three_flat_assignments() {
    RULE.assert_detects(
        r#"
mut settings = {}
$settings.theme = "dark"
$settings.font_size = 14
$settings.wrap = true
"#,
    );
}
