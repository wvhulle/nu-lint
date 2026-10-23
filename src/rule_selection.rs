use nu_lint_analysis::{
    config::{Config, LintLevel},
    rule::Rule,
};
use nu_lint_rules::{USED_RULES, groups::ALL_GROUPS};

use crate::{LintError, config_loading::validate_rule_levels};

fn rules_named_by(selection_name: &str) -> Option<Vec<&'static dyn Rule>> {
    ALL_GROUPS
        .iter()
        .find(|group| group.name == selection_name)
        .map(|group| group.rules.to_vec())
        .or_else(|| {
            USED_RULES
                .iter()
                .find(|rule| rule.id() == selection_name)
                .map(|rule| vec![*rule])
        })
}

fn selected_rule_ids(selection: &[String]) -> Result<Vec<&'static str>, LintError> {
    selection.iter().try_fold(Vec::new(), |mut rule_ids, name| {
        let rules = rules_named_by(name)
            .ok_or_else(|| LintError::SelectionDoesNotExist { name: name.clone() })?;
        rule_ids.extend(rules.iter().map(|rule| rule.id()));
        Ok(rule_ids)
    })
}

fn selected_level(rule: &dyn Rule, config: &Config) -> LintLevel {
    match config.get_lint_level(rule, ALL_GROUPS) {
        LintLevel::Off => LintLevel::Warning,
        configured_level => configured_level,
    }
}

pub fn restrict_config_to_selection(
    config: Config,
    selection: &[String],
) -> Result<Config, LintError> {
    let rule_ids = selected_rule_ids(selection)?;
    let rule_levels = USED_RULES
        .iter()
        .map(|rule| {
            let level = if rule_ids.contains(&rule.id()) {
                selected_level(*rule, &config)
            } else {
                LintLevel::Off
            };
            (rule.id().to_string(), level)
        })
        .collect();
    let restricted_config = Config {
        rules: rule_levels,
        ..config
    };
    validate_rule_levels(&restricted_config)?;
    Ok(restricted_config)
}

#[cfg(test)]
mod tests {
    use nu_lint_analysis::config::{Config, LintLevel};
    use nu_lint_rules::{USED_RULES, groups::ALL_GROUPS};

    use super::restrict_config_to_selection;
    use crate::LintError;

    fn selection(names: &[&str]) -> Vec<String> {
        names.iter().map(ToString::to_string).collect()
    }

    fn enabled_rule_ids(config: &Config) -> Vec<&'static str> {
        USED_RULES
            .iter()
            .filter(|rule| config.get_lint_level(**rule, ALL_GROUPS) != LintLevel::Off)
            .map(|rule| rule.id())
            .collect()
    }

    #[test]
    fn group_selects_exactly_its_rules() {
        let config = restrict_config_to_selection(
            Config::default(),
            &selection(&["migrate-deprecated-syntax"]),
        )
        .unwrap();
        let mut group_rule_ids: Vec<_> = ALL_GROUPS
            .iter()
            .find(|group| group.name == "migrate-deprecated-syntax")
            .unwrap()
            .rules
            .iter()
            .map(|rule| rule.id())
            .collect();
        group_rule_ids.sort_unstable();
        let mut enabled_ids = enabled_rule_ids(&config);
        enabled_ids.sort_unstable();
        assert_eq!(enabled_ids, group_rule_ids);
    }

    #[test]
    fn rule_id_selects_only_that_rule() {
        let config =
            restrict_config_to_selection(Config::default(), &selection(&["filter_to_where"]))
                .unwrap();
        assert_eq!(enabled_rule_ids(&config), ["filter_to_where"]);
    }

    #[test]
    fn selected_rule_that_is_off_by_default_runs_as_warning() {
        let config = restrict_config_to_selection(
            Config::default(),
            &selection(&["add_hat_external_commands"]),
        )
        .unwrap();
        assert_eq!(
            config.rules["add_hat_external_commands"],
            LintLevel::Warning
        );
    }

    #[test]
    fn selected_rule_keeps_configured_level() {
        let mut configured = Config::default();
        configured
            .rules
            .insert("filter_to_where".to_string(), LintLevel::Error);
        let config =
            restrict_config_to_selection(configured, &selection(&["filter_to_where"])).unwrap();
        assert_eq!(config.rules["filter_to_where"], LintLevel::Error);
    }

    #[test]
    fn unknown_name_is_rejected() {
        assert!(matches!(
            restrict_config_to_selection(Config::default(), &selection(&["nonsense"])),
            Err(LintError::SelectionDoesNotExist { name }) if name == "nonsense"
        ));
    }

    #[test]
    fn engine_reports_only_selected_rule() {
        let config =
            restrict_config_to_selection(Config::default(), &selection(&["filter_to_where"]))
                .unwrap();
        let violations =
            crate::lint_engine_with_all_rules(config).lint_str("[1 2] | filter {|x| $x > 1}");
        assert!(!violations.is_empty());
        assert!(
            violations
                .iter()
                .all(|violation| violation.rule_id.as_deref() == Some("filter_to_where"))
        );
    }
}
