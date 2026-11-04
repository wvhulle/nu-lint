use std::collections::HashMap;

use nu_lint_analysis::{Config, LintLevel, engine::LintEngine};

use super::RULE;
use crate::RULE_CATALOG;

#[test]
fn test_single_assignment() {
    RULE.assert_ignores(r#"$env.PATH = "/usr/bin""#);
}

#[test]
fn test_non_consecutive() {
    RULE.assert_ignores(
        r#"
$env.VAR1 = "value1"
let x = 5
$env.VAR2 = "value2"
"#,
    );
}

#[test]
fn test_nested_env_path() {
    // Nested $env paths are not handled by any rule (can't reassign $env)
    RULE.assert_ignores(
        r#"
$env.config.show_banner = false
$env.config.buffer_editor = "vim"
"#,
    );
}

#[test]
fn test_non_env_variable() {
    // Non-$env flat assignments are handled by use_record_spread
    RULE.assert_ignores(
        r#"
mut config = {}
$config.a = 1
$config.b = 2
"#,
    );
}

#[test]
fn test_no_conflict_with_use_record_spread() {
    // Flat non-$env should not trigger use_load_env
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
fn test_nested_non_env_not_detected() {
    // Nested non-$env should not trigger use_load_env
    RULE.assert_count(
        r#"
mut config = {}
$config.db.host = "localhost"
$config.db.port = 5432
"#,
        0,
    );
}

#[test]
fn config_level_applies_to_former_id() {
    let config = Config {
        rules: HashMap::from([("use_load_env".to_string(), LintLevel::Off)]),
        ..Default::default()
    };
    assert_eq!(config.get_lint_level(RULE, &[]), LintLevel::Off);
}

#[test]
fn ignore_comment_applies_to_former_id() {
    let engine = LintEngine::new(
        Config {
            rules: HashMap::from([(
                "env_assignments_to_load_env".to_string(),
                LintLevel::Warning,
            )]),
            ..Default::default()
        },
        RULE_CATALOG,
    );
    let assignments = "$env.A = 1\n$env.B = 2\n";
    let reports_rule = |source: &str| {
        engine
            .lint_str(source)
            .iter()
            .any(|violation| violation.rule_id.as_deref() == Some("env_assignments_to_load_env"))
    };
    assert!(reports_rule(assignments));
    assert!(!reports_rule(&format!(
        "# nu-lint-ignore-file: use_load_env\n{assignments}"
    )));
}
