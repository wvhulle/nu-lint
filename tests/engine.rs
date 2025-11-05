mod common;

use nu_lint::{LintEngine, config::Config};

#[test]
fn test_list_rules_returns_all_rules() {
    let config = Config::default();
    let engine = LintEngine::new(config);
    let rules: Vec<_> = engine.registry.all_rules().collect();

    assert!(!rules.is_empty());
    assert!(rules.iter().any(|r| r.id == "snake_case_variables"));
}

#[test]
fn test_explain_rule_exists() {
    let config = Config::default();
    let engine = LintEngine::new(config);
    let rule = engine.registry.get_rule("snake_case_variables");

    assert!(rule.is_some());
    let rule = rule.unwrap();
    assert_eq!(rule.id, "snake_case_variables");
    assert!(!rule.description.is_empty());
}

#[test]
fn test_explain_nonexistent_rule() {
    let config = Config::default();
    let engine = LintEngine::new(config);
    let rule = engine.registry.get_rule("NONEXISTENT");

    assert!(rule.is_none());
}
